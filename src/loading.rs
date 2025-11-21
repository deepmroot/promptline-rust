//! Loading indicator with witty messages
//!
//! Provides visual feedback during API calls

use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Witty loading messages to display while waiting
const LOADING_MESSAGES: &[&str] = &[
    "🤔 Thinking...",
    "⚙️  Processing your request...",
    "🧠 Analyzing code...",
    "✨ Brewing some wisdom...",
    "🔍 Reading between the lines...",
    "💭 Pondering the mysteries...",
    "🎯 Focusing neural pathways...",
    "⚡ Charging up the response...",
    "🚀 Launching thought rockets...",
    "🔮 Consulting the code oracle...",
    "🎪 Juggling bits and bytes...",
    "🌟 Channeling developer energy...",
];

/// Loading indicator with rotating messages
pub struct LoadingIndicator {
    /// Current message index
    current: Arc<AtomicUsize>,
    /// Whether the indicator is running
    running: Arc<AtomicBool>,
    /// Handle to the background task
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl LoadingIndicator {
    /// Create a new loading indicator
    pub fn new() -> Self {
        Self {
            current: Arc::new(AtomicUsize::new(0)),
            running: Arc::new(AtomicBool::new(false)),
            handle: None,
        }
    }

    /// Start the loading indicator
    pub fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);
        
        let current = Arc::clone(&self.current);
        let running = Arc::clone(&self.running);

        let handle = tokio::spawn(async move {
            while running.load(Ordering::SeqCst) {
                let idx = current.load(Ordering::SeqCst);
                let message = LOADING_MESSAGES[idx % LOADING_MESSAGES.len()];
                
                // Clear line and print message
                print!("\r{}", message);
                use std::io::Write;
                std::io::stdout().flush().ok();

                // Wait before next message
                sleep(Duration::from_secs(2)).await;

                // Move to next message
                current.store((idx + 1) % LOADING_MESSAGES.len(), Ordering::SeqCst);
            }

            // Clear the line when done
            print!("\r                                        \r");
            std::io::stdout().flush().ok();
        });

        self.handle = Some(handle);
    }

    /// Stop the loading indicator
    pub async fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        
        if let Some(handle) = self.handle.take() {
            handle.await.ok();
        }
    }

    /// Get a random loading message
    pub fn get_message() -> &'static str {
        use rand::Rng;
        let idx = rand::thread_rng().gen_range(0..LOADING_MESSAGES.len());
        LOADING_MESSAGES[idx]
    }
}

impl Default for LoadingIndicator {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for LoadingIndicator {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}
