// SPDX-License-Identifier: GPL-3.0-or-later

mod common;

use charond::domain::{CharonEvent, Mode};
use maiko::testing::EventMatcher;

use crate::common::setup;

#[tokio::test]
async fn test_key_press_emits_event() -> eyre::Result<()> {
    let mut ctx = setup().await?;
    ctx.sup.start().await?;
    ctx.state.set_mode(Mode::InApp);

    ctx.test.record().await;

    let event_id = ctx
        .test
        .send_as(&ctx.client, CharonEvent::SendText("test".into()))
        .await?;

    ctx.test
        .settle_on_event(EventMatcher::by_event(|e| {
            matches!(e, CharonEvent::TextSent)
        }))
        .await?;

    let chain = ctx.test.chain(event_id);
    println!("Mermaid:\n{}", chain.to_mermaid());

    assert!(
        chain
            .actors()
            .visited(&[&ctx.client, &ctx.typist, &ctx.writer, &ctx.telemetry])
    );

    let spy = ctx.test.actor(&ctx.typist);
    assert_eq!(2, spy.receiver_count());
    assert_eq!(1, spy.sender_count());
    assert_eq!(8, spy.outbound().with_label("HidReport").count());

    let spy = ctx.test.actor(&ctx.writer);
    assert_eq!(1, spy.receiver_count());
    assert_eq!(
        8,
        spy.inbound()
            .matching_event(|e| matches!(e, CharonEvent::HidReport(..)))
            .count()
    );

    // Ensure that Typist doesn't produce typing stats
    assert_eq!(0, ctx.metrics.lock().unwrap().key_events_counter);
    assert_eq!(0, ctx.metrics.lock().unwrap().key_to_report_time_counter);

    ctx.sup.stop().await?;
    Ok(())
}
