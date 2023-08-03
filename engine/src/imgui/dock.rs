use std::{collections::HashMap, ffi::CString};

use imgui::{sys::ImVec2, Direction, WindowToken};

pub struct DockSpace<'a, 'ui> {
    initialized: &'a mut bool,
    ui: &'ui imgui::Ui,
    label: String,
    split_nodes: Vec<SplitNode>,
    docked_windows: Vec<DockedWindow>,
}

struct SplitNode {
    label: String,
    direction: Direction,
    size_ratio: f32,
}

struct DockedWindow {
    label: String,
    split_node_label: String,
}

pub fn dockspace<'a, 'ui, 'b>(
    label: &'b str,
    ui: &'ui imgui::Ui,
    initialized: &'a mut bool,
) -> DockSpace<'a, 'ui> {
    DockSpace {
        initialized,
        ui,
        label: label.to_owned(),
        split_nodes: Vec::new(),
        docked_windows: Vec::new(),
    }
}

impl<'a, 'ui> DockSpace<'a, 'ui> {
    pub fn split_node(mut self, label: &str, direction: Direction, size_ratio: f32) -> Self {
        self.split_nodes.push(SplitNode {
            label: label.to_owned(),
            direction,
            size_ratio,
        });
        self
    }

    pub fn dock_window(mut self, label: &str, split_node_label: &str) -> Self {
        self.docked_windows.push(DockedWindow {
            label: label.to_owned(),
            split_node_label: split_node_label.to_owned(),
        });
        self
    }

    pub fn begin(self) -> Option<WindowToken<'ui>> {
        unsafe {
            use imgui::sys::*;

            // Setup host window
            let ui = self.ui;
            let viewport = igGetMainViewport();
            igSetNextWindowPos((*viewport).Pos, 0, imvec2(0.0, 0.0));
            igSetNextWindowSize((*viewport).Size, 0);
            igSetNextWindowViewport((*viewport).ID);
            let rounding_style = ui.push_style_var(imgui::StyleVar::WindowRounding(0.0));
            let border_style = ui.push_style_var(imgui::StyleVar::WindowBorderSize(0.0));
            let window_padding = ui.push_style_var(imgui::StyleVar::WindowPadding([0.0, 0.0]));
            let dockspace = ui
                .window(&self.label)
                .flags(imgui::WindowFlags::NO_DOCKING)
                .bring_to_front_on_focus(false)
                .nav_focus(false)
                .title_bar(false)
                .resizable(false)
                .movable(false)
                .collapsible(false)
                .draw_background(false)
                .menu_bar(true)
                .begin();
            window_padding.end();
            border_style.end();
            rounding_style.end();

            // DockSpace
            let dockspace_label = CString::new(self.label.clone()).unwrap();
            let mut dockspace_id = igGetIDStr(dockspace_label.as_ptr() as *const i8);
            igDockSpace(
                dockspace_id,
                imvec2(0.0, 0.0),
                ImGuiDockNodeFlags_PassthruCentralNode as _,
                std::ptr::null_mut(),
            );

            if !*self.initialized {
                *self.initialized = true;

                igDockBuilderRemoveNode(dockspace_id);
                igDockBuilderAddNode(
                    dockspace_id,
                    ImGuiDockNodeFlags_PassthruCentralNode as ImGuiDockNodeFlags
                        | ImGuiDockNodeFlags_DockSpace as ImGuiDockNodeFlags,
                );
                igDockBuilderSetNodeSize(dockspace_id, (*viewport).Size);

                let mut split_node_ids = HashMap::new();
                for split_node in self.split_nodes {
                    let id = igDockBuilderSplitNode(
                        dockspace_id,
                        split_node.direction as _,
                        split_node.size_ratio,
                        std::ptr::null_mut(),
                        &mut dockspace_id,
                    );
                    split_node_ids.insert(split_node.label, id);
                }

                for docked_window in self.docked_windows {
                    let node_id = if &docked_window.split_node_label == &self.label {
                        dockspace_id
                    } else {
                        split_node_ids[&docked_window.split_node_label]
                    };
                    let label_cstr = CString::new(docked_window.label).unwrap();
                    igDockBuilderDockWindow(label_cstr.as_ptr() as *const i8, node_id);
                }
            }

            dockspace
        }
    }
}

fn imvec2(x: f32, y: f32) -> ImVec2 {
    ImVec2 { x, y }
}
