Two threads write different variables that happen to live on the same 64-byte line. No logical conflict, but the line ping-pongs between cores anyway.

Fix: pad or align hot per-thread data to the line size. In Rust, `crossbeam_utils::CachePadded`.
