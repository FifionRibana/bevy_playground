use bevy::prelude::*;

use crate::hex::rendering::contour::terrain::{TerrainGenerationProgress, GenerationStage};

pub fn display_progress_ui(
    progress: Option<Res<TerrainGenerationProgress>>,
    mut contexts: Query<&mut bevy_egui::EguiContext>,
) {
    if let Some(progress) = progress {
        let Ok(mut ctx) = contexts.single_mut() else { return };
        
        bevy_egui::egui::Window::new("Génération du terrain")
            .show(ctx.get_mut(), |ui| {
                ui.label(format!("Étape: {:?}", progress.stage));
                ui.label(&progress.message);
                
                let progress_bar = bevy_egui::egui::ProgressBar::new(progress.progress)
                    .show_percentage();
                ui.add(progress_bar);
                
                match progress.stage {
                    GenerationStage::Complete => {
                        ui.label("✅ Génération terminée!");
                    }
                    _ => {
                        ui.label(format!("⏳ {:.1}%", progress.progress * 100.0));
                    }
                }
            });
    }
}