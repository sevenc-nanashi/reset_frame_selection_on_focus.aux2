mod watcher;
pub static EDIT_HANDLE: aviutl2::generic::GlobalEditHandle =
    aviutl2::generic::GlobalEditHandle::new();

#[aviutl2::plugin(GenericPlugin)]
struct ResetFrameSelectionOnFocusAux2 {
    watcher: std::sync::Arc<crate::watcher::Watcher>,
}

impl aviutl2::generic::GenericPlugin for ResetFrameSelectionOnFocusAux2 {
    fn new(_info: aviutl2::common::AviUtl2Info) -> aviutl2::common::AnyResult<Self> {
        let watcher = std::sync::Arc::new(crate::watcher::Watcher::new());
        Ok(Self {
            watcher: watcher.clone(),
        })
    }

    fn plugin_info(&self) -> aviutl2::generic::GenericPluginTable {
        aviutl2::generic::GenericPluginTable {
            name: "reset_frame_selection_on_focus.aux2".to_string(),
            information: format!(
                "Reset Frame Selection on Focus / v{} / https://github.com/sevenc-nanashi/reset_frame_selection_on_focus.aux2",
                env!("CARGO_PKG_VERSION")
            ),
        }
    }

    fn register(&mut self, registry: &mut aviutl2::generic::HostAppHandle) {
        EDIT_HANDLE.init(registry.create_edit_handle());
        registry.register_menus::<Self>();
    }

    fn event_change_focus_object(&mut self) {
        self.watcher.notify_focus_changed();
    }
}

#[aviutl2::generic::menus]
impl ResetFrameSelectionOnFocusAux2 {
    #[edit(name = "reset_frame_selection_on_focus.aux2\\通常モードに切り替え")]
    fn set_mode_enabled(&mut self) {
        self.watcher
            .set_mode(crate::watcher::WatcherThreadMode::Enabled);
    }

    #[edit(name = "reset_frame_selection_on_focus.aux2\\選択範囲を変更するまで無効化")]
    fn set_mode_disabled_for(&mut self) -> aviutl2::anyhow::Result<()> {
        let info = EDIT_HANDLE.get_edit_info();
        let start = info
            .select_range_start
            .ok_or_else(|| aviutl2::anyhow::anyhow!("No selection range"))?;
        let end = info
            .select_range_end
            .ok_or_else(|| aviutl2::anyhow::anyhow!("No selection range"))?;
        self.watcher
            .set_mode(crate::watcher::WatcherThreadMode::DisabledFor { start, end });
        Ok(())
    }

    #[edit(name = "reset_frame_selection_on_focus.aux2\\一時的に無効化")]
    fn set_mode_disabled(&mut self) {
        self.watcher
            .set_mode(crate::watcher::WatcherThreadMode::Disabled);
    }
}

aviutl2::register_generic_plugin!(ResetFrameSelectionOnFocusAux2);
