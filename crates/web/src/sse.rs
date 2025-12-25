/// Server-Sent Events (SSE) endpoint for real-time updates
/// Clients subscribe to this endpoint to receive data change notifications

use axum::{
    extract::State,
    response::sse::{Event, Sse},
};
use futures::stream::Stream;
use loaa_core::EventSender;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

/// SSE endpoint handler
/// Clients connect here to receive real-time data updates
pub async fn sse_handler(
    State(tx): State<EventSender>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    eprintln!("📺 New SSE client connected");

    // Subscribe to the broadcast channel
    let rx = tx.subscribe();

    // Convert broadcast receiver to a stream
    let stream = BroadcastStream::new(rx)
        .filter_map(|result| {
            match result {
                Ok(event) => {
                    // Serialize the event to JSON
                    match serde_json::to_string(&event) {
                        Ok(json) => {
                            eprintln!("📺 Sending SSE event: {}", json);
                            Some(Ok(Event::default().data(json)))
                        }
                        Err(e) => {
                            eprintln!("SSE serialization error: {}", e);
                            None
                        }
                    }
                }
                Err(e) => {
                    eprintln!("SSE broadcast error: {}", e);
                    None
                }
            }
        });

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(30))
            .text("keep-alive"),
    )
}
