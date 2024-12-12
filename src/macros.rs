#[macro_export]
macro_rules! runtime {
    ($label:expr, $block:block) => {{
        #[cfg(feature = "profiling")]
        {
            use std::time::Instant;
            let start = Instant::now();
            let result = { $block };
            let duration = start.elapsed();
            println!("[{}] Execution time: {:?}", $label, duration);
            result
        }
        #[cfg(not(feature = "profiling"))]
        {
            $block
        }
    }};
}
