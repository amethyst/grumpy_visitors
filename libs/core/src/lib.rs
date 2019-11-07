pub mod actions;
pub mod ecs;
pub mod math;
pub mod net;

#[macro_export]
macro_rules! profile_scope {
    ($string:expr) => {
        #[cfg(feature = "profiler")]
        let _profile_scope =
            thread_profiler::ProfileScope::new(format!("{}: {}", module_path!(), $string));
    };
}
