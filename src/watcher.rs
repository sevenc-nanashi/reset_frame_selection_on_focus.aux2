use aviutl2::{anyhow, tracing};

#[derive(Debug, Clone)]
enum WatcherThreadMessage {
    FocusChanged,
    Shutdown,
    ModeChange(WatcherThreadMode),
}

#[derive(Debug, Clone)]
pub enum WatcherThreadMode {
    Enabled,
    DisabledFor { start: usize, end: usize },
    Disabled,
}

pub struct Watcher {
    thread_handle: Option<std::thread::JoinHandle<()>>,
    sender: std::sync::mpsc::Sender<WatcherThreadMessage>,
}

impl Watcher {
    pub fn new() -> Self {
        let (sender, receiver) = std::sync::mpsc::channel::<WatcherThreadMessage>();

        let thread_handle = std::thread::spawn(move || {
            let mut mode = WatcherThreadMode::Enabled;
            loop {
                match receiver.recv() {
                    Ok(WatcherThreadMessage::FocusChanged) => {
                        Self::handle_focus_changed(&mut mode);
                    }
                    Ok(WatcherThreadMessage::Shutdown) => {
                        tracing::info!("Shutdown message received, exiting watcher thread.");
                        break;
                    }
                    Ok(WatcherThreadMessage::ModeChange(new_mode)) => {
                        mode = new_mode;
                        tracing::info!("Watcher thread mode changed to: {:?}", mode);
                    }
                    Err(_) => {
                        tracing::error!("Channel error, shutting down watcher thread.");
                        break;
                    }
                }
            }
        });

        Watcher {
            thread_handle: Some(thread_handle),
            sender,
        }
    }

    fn handle_focus_changed(mode: &mut WatcherThreadMode) {
        if !crate::EDIT_HANDLE.is_ready() {
            tracing::warn!("Edit handle is not ready, skipping focus change handling.");
            return;
        }
        let result = crate::EDIT_HANDLE
            .call_edit_section(|e| match mode {
                WatcherThreadMode::Enabled => {
                    tracing::debug!("Clearing selection range due to focus change.");
                    e.clear_select_range().map_err(anyhow::Error::from)
                }
                WatcherThreadMode::DisabledFor { start, end } => {
                    if Some(*start) == e.info.select_range_start
                        && Some(*end) == e.info.select_range_end
                    {
                        tracing::debug!("Selection range matches disabled range, skipping clear.");
                        Ok(())
                    } else {
                        tracing::debug!(
                            "Selection range does not match disabled range, clearing selection."
                        );
                        *mode = WatcherThreadMode::Enabled;
                        e.clear_select_range().map_err(anyhow::Error::from)
                    }
                }
                WatcherThreadMode::Disabled => {
                    tracing::debug!("Selection range clearing is disabled.");
                    Ok(())
                }
            })
            .map_err(anyhow::Error::from);
        if let Err(e) = result {
            tracing::error!("Failed to clear selection range: {:?}", e);
        }
    }

    pub fn set_mode(&self, mode: WatcherThreadMode) {
        let _ = self.sender.send(WatcherThreadMessage::ModeChange(mode));
    }

    pub fn notify_focus_changed(&self) {
        let _ = self.sender.send(WatcherThreadMessage::FocusChanged);
    }

    pub fn shutdown(&self) {
        let _ = self.sender.send(WatcherThreadMessage::Shutdown);
    }
}
impl Drop for Watcher {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }
    }
}
