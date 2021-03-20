use std::sync::mpsc;

use eframe::{
    egui::{self, Align, Button, CentralPanel, Layout, SidePanel},
    epi,
};
use es_manager::{
    version::{RepoVersion, Version},
    Instance,
};

use anyhow::*;

#[derive(Debug)]
pub struct App {
    instances: Vec<Instance>,
    state: AppState,
    channel: mpsc::Receiver<Instance>,
    sender: mpsc::Sender<Instance>,
}

impl App {
    pub fn new(instances: Vec<Instance>) -> Self {
        let (sender, channel) = mpsc::channel();
        Self {
            instances,
            state: AppState::None,
            channel,
            sender,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    None,
    Creating { name: String, version: RepoVersion },
    Editing(usize),
}

impl Default for AppState {
    fn default() -> Self {
        Self::None
    }
}

impl epi::App for App {
    fn name(&self) -> &str {
        "ESManager"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _: &mut epi::Frame<'_>) {
        let App {
            instances,
            state,
            channel,
            sender,
        } = self;

        while let Ok(instance) = channel.try_recv() {
            instances.push(instance);
        }
        instances.sort_by(|x, y| x.name.cmp(&y.name));

        SidePanel::left("left", 200.0).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Instances");
                ui.with_layout(Layout::default().with_cross_align(Align::Max), |ui| {
                    if ui.small_button("+").clicked() {
                        *state = AppState::Creating {
                            name: Default::default(),
                            version: Default::default()
                        };
                    }
                });
            });
            for (idx, instance) in instances.iter().enumerate() {
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.selectable_label(*state == AppState::Editing(idx), &instance.name).clicked() {
                        *state = AppState::Editing(idx)
                    }
                    ui.with_layout(Layout::default().with_cross_align(Align::Max), |ui| {
                        if ui.button("Launch").clicked() {
                            let instance = instance.clone();
                            std::thread::spawn(move || instance.launch().unwrap());
                        }
                    });
                });
            }
        });
        CentralPanel::default().show(ctx, |ui| {
            match state {
                AppState::None => {}
                AppState::Creating {
                    ref mut name,
                    ref mut version,
                } => {
                    let mut created = false;
                    ui.horizontal(|ui| {
                        ui.heading("Add Instance");
                        ui.with_layout(Layout::default().with_cross_align(Align::Max), |ui| {
                            if ui
                                .add(
                                    Button::new("Create")
                                        .enabled(!name.is_empty() && version.is_valid()),
                                )
                                .clicked()
                            {
                                let name = std::mem::take(name);
                                let version = std::mem::take(version);
                                let sender = sender.clone();
                                std::thread::Builder::new()
                                    .name(name.clone())
                                    .spawn(move || {
                                        let mut res = Err(anyhow!(""));
                                        while let Err(_) = res {
                                            res = Instance::create(name.clone(), version.clone());
                                        }
                                        sender.send(res.unwrap()).unwrap();
                                    })
                                    .unwrap();
                                created = true;
                            }
                        })
                    });
                    ui.horizontal(|ui| {
                        ui.label("Name: ");
                        ui.text_edit_singleline(name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Repo: ");
                        ui.text_edit_singleline(&mut version.repo)
                    });

                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut version.version,
                            Version::BranchHead("master".to_string()),
                            "Branch",
                        );
                        if let Version::BranchHead(ref mut branch) = &mut version.version {
                            ui.text_edit_singleline(branch);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(
                            &mut version.version,
                            Version::Commit("".to_string()),
                            "Commit",
                        );
                        if let Version::Commit(ref mut commit) = &mut version.version {
                            ui.text_edit_singleline(commit);
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.radio_value(&mut version.version, Version::Tag("".to_string()), "Tag");
                        if let Version::Tag(ref mut tag) = &mut version.version {
                            ui.text_edit_singleline(tag);
                        }
                    });
                    if created {
                        *state = AppState::None;
                    }
                }
                AppState::Editing(_) => {}
            };
        });
    }
}
