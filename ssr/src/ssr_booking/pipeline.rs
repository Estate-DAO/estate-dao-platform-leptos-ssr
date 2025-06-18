use crate::utils::notifier::{self, Notifier};
use crate::utils::notifier_event::{NotifierEvent, NotifierEventType};
use crate::utils::uuidv7;
use async_trait::async_trait;
use chrono::Utc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, instrument, span, Instrument, Level};

use super::{SSRBookingPipelineStep, ServerSideBookingEvent};

#[derive(Debug, Clone, PartialEq)]
pub enum PipelineDecision {
    Run,
    Skip,
    Abort(String),
}

// --------------------------
// Traits
// --------------------------

/// Validation always takes &self.
#[async_trait]
pub trait PipelineValidator: Send + Sync {
    async fn validate(&self, event: &ServerSideBookingEvent) -> Result<PipelineDecision, String>;
}

/// Execution is stateless so it does not need &self.
#[async_trait]
pub trait PipelineExecutor: Send + Sync {
    async fn execute(event: ServerSideBookingEvent, notifier: Option<&Notifier>) -> Result<ServerSideBookingEvent, String>;
}

// --------------------------
// Process the pipeline of steps in order, optionally publishing events via `notifier`.
// --------------------------

/// Process the pipeline of steps in order, optionally publishing events via `notifier`.
#[instrument(skip(steps, notifier), fields(correlation_id))]
pub async fn process_pipeline(
    event: ServerSideBookingEvent,
    steps: &[SSRBookingPipelineStep],
    notifier: Option<&Notifier>,
) -> Result<ServerSideBookingEvent, String> {
    let mut current_event = event;

    // Generate a correlation_id for this pipeline run.
    let correlation_id = uuidv7::create();
    tracing::Span::current().record("correlation_id", &correlation_id.as_str());
    info!("process_pipeline started");

    // 1. Notify pipeline start
    if let Some(n) = notifier {
        // let span = span!(Level::INFO, "notify_pipeline_start");
        // let _enter = span.enter();

        let pipeline_start_event = NotifierEvent {
            event_id: uuidv7::create(),
            correlation_id: correlation_id.clone(),
            timestamp: Utc::now(),
            order_id: current_event.order_id.clone(),
            step_name: None,
            event_type: NotifierEventType::OnPipelineStart,
            email: current_event.user_email.clone(),
        };
        info!("pipeline_start_event = {pipeline_start_event:#?}");

        n.notify(pipeline_start_event).await;
    }

    // 2. Iterate over steps
    for step in steps {
        // For logging or event purposes, let's define a step name
        let step_name = step.to_string();
        let step_span = span!(Level::INFO, "pipeline_step", name = %step_name);
        let _enter = step_span.enter();

        // We first validate
        let decision = step.validate(&current_event).await?;

        match decision {
            PipelineDecision::Abort(reason) => {
                info!(status = "aborted", reason = %reason, "Pipeline step aborted");
                // Publish OnPipelineAbort
                if let Some(n) = notifier {
                    let abort_event = NotifierEvent {
                        event_id: uuidv7::create(),
                        correlation_id: correlation_id.clone(),
                        timestamp: Utc::now(),
                        order_id: current_event.order_id.clone(),
                        step_name: Some(step_name.clone()),
                        event_type: NotifierEventType::OnPipelineAbort,
                        email: current_event.user_email.clone(),
                    };
                    n.notify(abort_event).await;
                }

                return Err(format!("Pipeline aborted: {}", reason));
            }
            PipelineDecision::Skip => {
                info!(status = "skipped", "Pipeline step skipped");
                // Publish OnStepSkipped
                if let Some(n) = notifier {
                    let skipped_event = NotifierEvent {
                        event_id: uuidv7::create(),
                        correlation_id: correlation_id.clone(),
                        timestamp: Utc::now(),
                        order_id: current_event.order_id.clone(),
                        step_name: Some(step_name.clone()),
                        event_type: NotifierEventType::OnStepSkipped,
                        email: current_event.user_email.clone(),
                    };
                    n.notify(skipped_event).await;
                }

                // Do not execute the step
                continue;
            }
            PipelineDecision::Run => {
                info!(status = "running", "Pipeline step starting");
                // Publish OnStepStart
                if let Some(n) = notifier {
                    let start_event = NotifierEvent {
                        event_id: uuidv7::create(),
                        correlation_id: correlation_id.clone(),
                        timestamp: Utc::now(),
                        order_id: current_event.order_id.clone(),
                        step_name: Some(step_name.clone()),
                        event_type: NotifierEventType::OnStepStart,
                        email: current_event.user_email.clone(),
                    };
                    n.notify(start_event).await;
                }

                // Actually run the step
                current_event = step.execute(current_event, notifier).await?;

                info!(status = "completed", "Pipeline step completed");
                // Publish OnStepCompleted
                if let Some(n) = notifier {
                    let completed_event = NotifierEvent {
                        event_id: uuidv7::create(),
                        correlation_id: correlation_id.clone(),
                        timestamp: Utc::now(),
                        order_id: current_event.order_id.clone(),
                        step_name: Some(step_name.clone()),
                        event_type: NotifierEventType::OnStepCompleted,
                        email: current_event.user_email.clone(),
                    };
                    n.notify(completed_event).await;
                }
            }
        }
    }

    let notifier_span = span!(Level::DEBUG, "notifier");
    {
        let _nspan = notifier_span.enter();
        // 3. If all steps succeed, publish OnPipelineEnd
        if let Some(n) = notifier {
            let end_event = NotifierEvent {
                event_id: uuidv7::create(),
                correlation_id,
                timestamp: Utc::now(),
                order_id: current_event.order_id.clone(),
                step_name: None,
                event_type: NotifierEventType::OnPipelineEnd,
                email: current_event.user_email.clone(),
            };
            debug!("NotifierEvent = {end_event:#?}");
            n.notify(end_event).await;
        }
    }

    // this is only for local testing purpose of concurrency of the pipeline.
    #[cfg(feature = "mock-pipeline")]
    tokio::time::sleep(Duration::from_millis(4000)).await;

    info!("process_pipeline completed");
    Ok(current_event)
}
