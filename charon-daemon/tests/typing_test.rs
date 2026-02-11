// SPDX-License-Identifier: GPL-3.0-or-later

mod common;

use std::time::Duration;

use charond::domain::CharonEvent;

use crate::common::setup;

#[tokio::test]
async fn test_key_press_emits_event() -> eyre::Result<()> {
    let mut ctx = setup().await?;
    ctx.sup.start().await?;
    ctx.test.start_recording().await;

    let event_id = ctx
        .test
        .send_as(&ctx.client, CharonEvent::SendText("test".into()))
        .await?;

    tokio::time::sleep(Duration::from_millis(2000)).await;
    ctx.test.stop_recording().await;

    let chain = ctx.test.chain(event_id);
    println!("Mermaid:\n{}", chain.to_mermaid());

    assert!(chain.actors().visited(&[&ctx.client, &ctx.typist]));

    ctx.sup.stop().await?;
    Ok(())
}
