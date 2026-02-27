// SPDX-License-Identifier: GPL-3.0-or-later

mod common;

use std::time::Duration;

use evdev::KeyCode;

use crate::common::setup;

#[tokio::test]
async fn test_key_press_emits_event() -> eyre::Result<()> {
    let mut ctx = setup().await?;

    println!("{}", ctx.sup.to_mermaid());

    ctx.sup.start().await?;

    ctx.test.record().await;
    ctx.keyboard.key_press(KeyCode::KEY_S).await;
    ctx.keyboard.drain().await;
    ctx.test.settle().await;

    let spy = ctx.test.actor(&ctx.scanner);
    assert_eq!(2, spy.receiver_count());
    let event = spy.last_sent().unwrap();
    let chain = ctx.test.chain(event.id());

    println!("\nActor chain for KeyPress S:");
    println!("{:?}", chain.actors().all());

    println!("\nMermaid diagram:");
    println!("{}", chain.to_mermaid());

    print!("\nEvent chain for KeyPress S: {}", chain.to_string_tree());

    assert!(chain.events().contains(event.id()));
    assert_eq!(chain.actors().path_count(), 2);
    assert!(chain.actors().segment(&[&ctx.scanner, &ctx.telemetry]));
    assert!(
        chain
            .actors()
            .exact(&[&ctx.scanner, &ctx.pipeline, &ctx.writer, &ctx.telemetry])
    );

    assert!(
        chain
            .events()
            .segment(&["KeyPress", "HidReport", "ReportSent"])
    );

    ctx.sup.stop().await?;
    Ok(())
}

#[tokio::test]
async fn test_ctrl_q_shortcut_flow() -> eyre::Result<()> {
    tokio::time::timeout(Duration::from_secs(5), async {
        let mut ctx = setup().await?;

        ctx.sup.start().await?;
        ctx.test.record().await;

        // Step 1: Press Ctrl - this flows through to the host
        ctx.keyboard.key_press(KeyCode::KEY_LEFTCTRL).await;
        ctx.keyboard.drain().await;

        // Step 2: Press Q while Ctrl is held - this triggers shutdown
        ctx.keyboard.key_press(KeyCode::KEY_Q).await;
        ctx.keyboard.drain().await;

        ctx.test.settle().await;

        // Scanner should have sent 2 distinct events: Ctrl and Q
        let scanner_spy = ctx.test.actor(&ctx.scanner);
        assert_eq!(
            scanner_spy.receiver_count(),
            2,
            "Expected 2 events from scanner (Ctrl and Q)"
        );

        // Get unique events by collecting and deduplicating by ID
        let unique_events: Vec<_> = scanner_spy.outbound().collect();
        let ctrl_event = &unique_events[0];
        let q_event = &unique_events[1];

        // Analyze Ctrl key chain - should flow all the way through
        let ctrl_chain = ctx.test.chain(ctrl_event.id());

        println!("\n=== Ctrl Key Chain ===");
        println!("{}", ctrl_chain.to_string_tree());
        println!("Mermaid:\n{}", ctrl_chain.to_mermaid());
        println!("Paths: {:?}", ctrl_chain.actors().paths());

        // Ctrl flows: Scanner -> Pipeline -> Writer -> Telemetry
        //             Scanner -> Telemetry (direct via KeyInput topic)
        assert_eq!(ctrl_chain.actors().path_count(), 2);
        assert!(
            ctrl_chain
                .actors()
                .exact(&[&ctx.scanner, &ctx.pipeline, &ctx.writer, &ctx.telemetry])
        );
        assert!(ctrl_chain.actors().exact(&[&ctx.scanner, &ctx.telemetry]));

        // Ctrl generates HidReport and ReportSent
        assert!(
            ctrl_chain
                .events()
                .segment(&["KeyPress", "HidReport", "ReportSent"])
        );

        // Analyze Q key chain - should be intercepted by SystemShortcutProcessor
        let q_chain = ctx.test.chain(q_event.id());

        println!("\n=== Q Key Chain (Ctrl+Q shortcut) ===");
        println!("{}", q_chain.to_string_tree());
        println!("Mermaid:\n{}", q_chain.to_mermaid());
        println!("Paths: {:?}", q_chain.actors().paths());

        // Q is intercepted by SystemShortcutProcessor which calls stop_runtime().
        // This triggers immediate shutdown, so Telemetry may not process Q before
        // the runtime is cancelled.
        assert!(q_chain.actors().visited(&[&ctx.scanner, &ctx.pipeline]));

        // Verify all actors in the chain have stopped (Ctrl+Q triggered shutdown)
        for actor in q_chain.actors().all() {
            tracing::info!("Checking on {actor} whether it's closed");
            assert!(ctx.test.actor(actor).is_stopped())
        }

        Ok(())
    })
    .await
    .expect("test timed out — supervisor may not have stopped")
}
