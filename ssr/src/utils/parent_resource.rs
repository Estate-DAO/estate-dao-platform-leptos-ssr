// use futures::{Future, StreamExt};
// use leptos::prelude::*;
// use serde::{de::DeserializeOwned, Deserialize, Serialize};
// use std::time::Duration;
// use tokio::time::sleep;

// // Wrapper for PartialEq that always returns false
// /// this is currently only used for resources
// /// this does not provide a sane implementation of PartialEq
// #[derive(Clone, Serialize, Deserialize)]
// pub struct MockPartialEq<T>(pub T);

// impl<T> PartialEq for MockPartialEq<T> {
//     fn eq(&self, _: &Self) -> bool {
//         false
//     }
// }

// #[derive(Clone)]
// pub struct ParentResource<S: 'static + Clone + Send + Sync, T: 'static + Clone + Send + Sync>(
//     pub Resource<S, T>,
// );

// impl<S: 'static + Clone + Send + Sync, T: 'static + Clone + Send + Sync> ParentResource<S, T> {
//     /// Derive another resource that depends on this resource
//     /// Note: the source is not memoized like it is for resources
//     pub fn derive<
//         DS: 'static + Clone + Send + Sync,
//         DT: 'static + Send + Sync + Serialize + for<'de> Deserialize<'de>,
//         F: Future<Output = DT> + 'static + Send,
//     >(
//         &self,
//         source: impl Fn() -> DS + 'static,
//         fetcher: impl Fn(T, DS) -> F + Clone + 'static,
//     ) -> Resource<MockPartialEq<DS>, DT> {
//         let parent = self.0;
//         let tracker = Memo::new(move |prev| {
//             let prev: bool = prev.copied().unwrap_or_default();
//             let parent_is_none = parent.with(|p| p.is_none());
//             // If parent is none -> Resource is reloading
//             if parent_is_none {
//                 !prev
//             // resource is loaded -> we were already waiting for it, so we don't need to reload
//             } else {
//                 prev
//             }
//         });

//         let parent_signal = parent.read();
//         Resource::new(
//             move || {
//                 tracker();
//                 MockPartialEq(source())
//             },
//             move |st| {
//                 let mut val_st = parent_signal.to_stream();
//                 let fetcher = fetcher.clone();
//                 async move {
//                     let val = loop {
//                         let res = val_st.next().await.expect("Signal stream ended?!");
//                         if let Some(val) = res {
//                             break val;
//                         }
//                     };
//                     fetcher(val, st.0).await
//                 }
//             },
//         )
//     }

//     pub async fn wait_untracked(&self) -> T {
//         let parent = self.0;
//         let parent_signal = parent.read();
//         let mut val_st = parent_signal.to_stream();
//         loop {
//             let res = val_st.next().await.expect("Signal stream ended?!");
//             if let Some(val) = res {
//                 return val;
//             }
//         }
//     }
// }

// pub fn derive<S, T, F, Fu>(
//     parent: Resource<S, T>,
//     derive_fn: F,
// ) -> Resource<Option<S>, Option<T>>
// where
//     S: Clone + Send + Sync + 'static,
//     T: Clone + Send + Sync + 'static,
//     F: Fn(S) -> Fu + Clone + Send + Sync + 'static,
//     Fu: std::future::Future<Output = T> + Send + 'static,
// {
//     Resource::new(
//         move || parent.get(),
//         async move |parent_value: Option<S>| {
//             if let Some(parent_value) = parent_value {
//                 Some(derive_fn(parent_value).await)
//             } else {
//                 None
//             }
//         },
//     )
// }

// pub fn derive_s<S, T, F, Fu>(
//     parent: Resource<S, T>,
//     derive_fn: F,
// ) -> Resource<Option<S>, Option<T>>
// where
//     S: Clone + Send + Sync + 'static,
//     T: Clone + Send + Sync + Serialize + DeserializeOwned + 'static,
//     F: Fn(S) -> Fu + Clone + Send + Sync + 'static,
//     Fu: std::future::Future<Output = T> + Send + 'static,
// {
//     Resource::new(
//         move || parent.get(),
//         async move |parent_value: Option<S>| {
//             if let Some(parent_value) = parent_value {
//                 Some(derive_fn(parent_value).await)
//             } else {
//                 None
//             }
//         },
//     )
// }

// pub async fn wait_untracked<S, T>(parent: Resource<S, T>) -> T
// where
//     S: Clone + Send + Sync + 'static,
//     T: Clone + Send + Sync + 'static,
// {
//     loop {
//         if let Some(val) = parent.get_untracked() {
//             return val;
//         }
//         sleep(Duration::from_millis(5)).await;
//     }
// }
