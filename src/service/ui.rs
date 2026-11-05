use super::*;
use crate::game_ui::{Document, MAX_BYTES, Operation};
impl Service {
    pub(super) fn ui_command(
        &mut self,
        operation: &Operation,
        revision: Option<u64>,
    ) -> Result<Value> {
        self.game_ui.guard(revision)?;
        match operation {
            Operation::Inspect => {}
            Operation::Replace { document } => self.game_ui.replace(*document.clone())?,
            Operation::Edit { edits } => self.game_ui.edit(edits)?,
            Operation::Undo => self.game_ui.history(false)?,
            Operation::Redo => self.game_ui.history(true)?,
            Operation::Load { path } => {
                let source = self.resolve(path)?;
                if !std::fs::metadata(&source).map_err(io_error)?.is_file() {
                    return Err(invalid("UI document must be a regular file"));
                }
                let file = std::fs::File::open(source).map_err(io_error)?;
                if !file.metadata().map_err(io_error)?.is_file() {
                    return Err(invalid("UI document must be a regular file"));
                }
                let mut bytes = vec![];
                file.take((MAX_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)
                    .map_err(io_error)?;
                if bytes.len() > MAX_BYTES {
                    return Err(invalid("UI file exceeds256 KiB"));
                }
                let d: Document = serde_json::from_slice(&bytes)
                    .map_err(|e| invalid(format!("Invalid UI document: {e}")))?;
                self.game_ui.replace(d)?;
            }
            Operation::Save { path } => {
                let bytes = serde_json::to_vec(self.game_ui.document()).map_err(io_error)?;
                self.write(path, &bytes)?;
            }
        }
        // Read commands return the document. Mutations return a compact receipt.
        Ok(if matches!(operation, Operation::Inspect) {
            self.game_ui.inspect()
        } else {
            json!({"ui_revision":self.game_ui.revision()})
        })
    }
}
