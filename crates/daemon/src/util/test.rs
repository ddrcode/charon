#[macro_use]
pub(crate) mod macros {

    macro_rules! with_lock {
        ($mutex:expr, |$lock:ident| $body:block) => {{
            let $lock = $mutex.lock().await;
            $body
        }};
    }

    macro_rules! assert_event_matches {
        ($rx:expr, $pat:pat $(if $guard:expr)? $(,)?) => {{
            let event = $rx.recv().await.expect("Expected an event");
            match &event.payload {
                $pat $(if $guard)? => {},
                other => panic!(
                    "Unexpected event: {:?}, expected pattern: {}",
                    other,
                    stringify!($pat)
                ),
            }
        }};
    }

    pub(crate) use {assert_event_matches, with_lock};
}
