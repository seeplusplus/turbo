use anyhow::Result;
use turbo_tasks::{primitives::StringVc, ValueToString, ValueToStringVc};
use turbo_tasks_fs::{File, FileContent, FileContentVc, FileSystemPathVc};
use turbopack_core::{
    asset::{Asset, AssetVc},
    chunk::{
        chunk_content, Chunk, ChunkContentResultVc, ChunkContext, ChunkContextVc,
        ChunkGroupReferenceVc, ChunkPlaceableVc, ChunkReferenceVc, ChunkVc, ChunkingContextVc,
        ModuleIdVc,
    },
    reference::AssetReferencesVc,
};

#[turbo_tasks::value(Chunk, Asset, ValueToString)]
pub struct EcmascriptChunk {
    context: ChunkingContextVc,
    /// must implement [EcmascriptChunkPlaceable] too
    entry: AssetVc,
}

#[turbo_tasks::value_impl]
impl EcmascriptChunkVc {
    #[turbo_tasks::function]
    pub fn new(context: ChunkingContextVc, entry: AssetVc) -> Self {
        Self::cell(EcmascriptChunk { context, entry })
    }
}

#[turbo_tasks::function]
fn chunk_context(_context: ChunkingContextVc) -> ChunkContextVc {
    EcmascriptChunkContextVc::cell(EcmascriptChunkContext {}).into()
}

#[turbo_tasks::function]
async fn ecmascript_chunk_content(
    context: ChunkingContextVc,
    entry: AssetVc,
) -> Result<ChunkContentResultVc> {
    chunk_content(context, entry, EcmascriptChunkPlaceableVc::resolve_from).await
}

#[turbo_tasks::value_impl]
impl Chunk for EcmascriptChunk {}

#[turbo_tasks::value_impl]
impl ValueToString for EcmascriptChunk {
    #[turbo_tasks::function]
    async fn to_string(&self) -> Result<StringVc> {
        Ok(StringVc::cell(format!(
            "chunk {}",
            self.entry.path().to_string().await?
        )))
    }
}

#[turbo_tasks::value_impl]
impl Asset for EcmascriptChunk {
    #[turbo_tasks::function]
    fn path(&self) -> FileSystemPathVc {
        self.context.as_chunk_path(self.entry.path())
    }

    #[turbo_tasks::function]
    async fn content(&self) -> Result<FileContentVc> {
        let content = ecmascript_chunk_content(self.context, self.entry).await?;
        let c_context = chunk_context(self.context);
        let mut code = String::new();
        for chunk_item in content.chunk_items.iter() {
            let content = &chunk_item.content(c_context, self.context).await?;
            code += &content;
            code += "\n\n";
        }
        Ok(FileContent::Content(File::from_source(code)).into())
    }

    #[turbo_tasks::function]
    async fn references(&self) -> Result<AssetReferencesVc> {
        let content = ecmascript_chunk_content(self.context, self.entry).await?;
        let mut references = Vec::new();
        for r in content.external_asset_references.iter() {
            references.push(*r);
        }
        for chunk in content.chunks.iter() {
            references.push(ChunkReferenceVc::new_parallel(*chunk).into());
        }
        for chunk_group in content.async_chunk_groups.iter() {
            references.push(ChunkGroupReferenceVc::new(*chunk_group).into());
        }
        Ok(AssetReferencesVc::cell(references))
    }
}

#[turbo_tasks::value(ChunkContext)]
pub struct EcmascriptChunkContext {}

#[turbo_tasks::value_impl]
impl ChunkContext for EcmascriptChunkContext {}

#[turbo_tasks::value_impl]
impl EcmascriptChunkContextVc {
    #[turbo_tasks::function]
    fn id(self, _placeable: EcmascriptChunkPlaceableVc) -> ModuleIdVc {
        todo!()
    }
}

#[turbo_tasks::value_trait]
pub trait EcmascriptChunkPlaceable: ValueToString + ChunkPlaceable {}
