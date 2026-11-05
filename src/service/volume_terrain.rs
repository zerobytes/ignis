use super::*;
impl Service {
    pub(super) fn volume_terrain_gpu_command(&mut self, command: &Command) -> Result<Value> {
        let (document, grid) = match command {
            Command::QueryVolumeTerrainGpu { document, grid } => (document.as_ref(), grid),
            Command::PublishVolumeTerrainGpu { publication } => {
                if self.engine.playing {
                    return Err(invalid("Pause before publishing density terrain"));
                }
                (&publication.document, &publication.grid)
            }
            _ => return Err(invalid("Expected GPU density operation")),
        };
        document.validate()?;
        if self.capture_renderer.is_none() {
            let instance = wgpu::Instance::new(&Default::default());
            self.capture_renderer = Some(
                pollster::block_on(Renderer::new(&instance, None)).map_err(|e| Error {
                    code: "graphics_unavailable",
                    message: e.to_string(),
                })?,
            );
        }
        let r = self.capture_renderer.as_ref().unwrap();
        let gpu = self
            .volume_gpu
            .get_or_insert_with(|| crate::volume_terrain::gpu::Gpu::new(&r.device));
        let c = gpu.compile(&r.device, &r.queue, document, grid)?;
        let report = c.report.clone();
        if let Command::PublishVolumeTerrainGpu { publication } = command {
            if c.geometry.triangles.is_empty() {
                return Err(invalid(
                    "Density grid is empty; inspect or choose another region before publishing",
                ));
            }
            let collision = publication.collision;
            let (asset, instance, _) =
                crate::volume_terrain::publication_from((**publication).clone(), c);
            let id = instance.id.clone();
            let mut edits = vec![
                crate::engine::SceneEdit::Mesh {
                    edit: crate::meshes::Edit::PutAsset { asset },
                },
                crate::engine::SceneEdit::Mesh {
                    edit: crate::meshes::Edit::PutInstance { instance },
                },
            ];
            if let Some(enabled) = collision {
                edits.push(crate::engine::SceneEdit::SetMeshCollision {
                    instance: id.clone(),
                    enabled,
                });
            }
            let publication_start = std::time::Instant::now();
            let mut result = self.engine.execute(Command::EditScene { edits }.into())?;
            let publication_ms = publication_start.elapsed().as_secs_f64() * 1000.;
            result["terrain"] = report;
            result["terrain"]["timing_wall_ms"]["scene_publication_including_collision"] =
                json!(publication_ms);
            result["source_embedded_in_scene"] = json!(false);
            result["visual_only"] = json!(!self.engine.scene.collision_meshes.contains(&id));
            Ok(result)
        } else {
            Ok(report)
        }
    }
}
