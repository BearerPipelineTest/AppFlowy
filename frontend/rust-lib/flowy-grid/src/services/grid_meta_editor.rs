use bytes::Bytes;
use flowy_collaboration::client_grid::{GridBlockMetaChange, GridBlockMetaPad};
use flowy_collaboration::entities::revision::Revision;
use flowy_collaboration::util::make_delta_from_revisions;
use flowy_error::{FlowyError, FlowyResult};
use flowy_grid_data_model::entities::{RowMeta, RowMetaChangeset};
use flowy_sync::{RevisionCloudService, RevisionCompactor, RevisionManager, RevisionObjectBuilder};
use lib_infra::future::FutureResult;
use lib_ot::core::PlainTextAttributes;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ClientGridBlockMetaEditor {
    user_id: String,
    pub block_id: String,
    meta_pad: Arc<RwLock<GridBlockMetaPad>>,
    rev_manager: Arc<RevisionManager>,
}

impl ClientGridBlockMetaEditor {
    pub async fn new(
        user_id: &str,
        token: &str,
        block_id: &str,
        mut rev_manager: RevisionManager,
    ) -> FlowyResult<Self> {
        let cloud = Arc::new(GridBlockMetaRevisionCloudService {
            token: token.to_owned(),
        });
        let block_meta_pad = rev_manager.load::<GridBlockMetaPadBuilder>(Some(cloud)).await?;
        let meta_pad = Arc::new(RwLock::new(block_meta_pad));
        let rev_manager = Arc::new(rev_manager);
        let user_id = user_id.to_owned();
        let block_id = block_id.to_owned();
        Ok(Self {
            user_id,
            block_id,
            meta_pad,
            rev_manager,
        })
    }

    async fn create_row(&self) -> FlowyResult<()> {
        let row = RowMeta::new(&self.block_id, vec![]);
        let _ = self.modify(|pad| Ok(pad.add_row(row)?)).await?;
        Ok(())
    }

    pub async fn delete_rows(&self, ids: Vec<String>) -> FlowyResult<()> {
        let _ = self.modify(|pad| Ok(pad.delete_rows(&ids)?)).await?;
        Ok(())
    }

    pub async fn update_row(&self, changeset: RowMetaChangeset) -> FlowyResult<()> {
        let _ = self.modify(|pad| Ok(pad.update_row(changeset)?)).await?;
        Ok(())
    }

    pub async fn get_rows(&self, row_ids: Vec<String>) -> FlowyResult<Vec<RowMeta>> {
        let rows = self.meta_pad.read().await.get_rows(row_ids)?;
        Ok(rows)
    }

    async fn modify<F>(&self, f: F) -> FlowyResult<()>
    where
        F: for<'a> FnOnce(&'a mut GridBlockMetaPad) -> FlowyResult<Option<GridBlockMetaChange>>,
    {
        let mut write_guard = self.meta_pad.write().await;
        match f(&mut *write_guard)? {
            None => {}
            Some(change) => {
                let _ = self.apply_change(change).await?;
            }
        }
        Ok(())
    }

    async fn apply_change(&self, change: GridBlockMetaChange) -> FlowyResult<()> {
        let GridBlockMetaChange { delta, md5 } = change;
        let user_id = self.user_id.clone();
        let (base_rev_id, rev_id) = self.rev_manager.next_rev_id_pair();
        let delta_data = delta.to_bytes();
        let revision = Revision::new(
            &self.rev_manager.object_id,
            base_rev_id,
            rev_id,
            delta_data,
            &user_id,
            md5,
        );
        let _ = self
            .rev_manager
            .add_local_revision(&revision, Box::new(GridBlockMetaRevisionCompactor()))
            .await?;
        Ok(())
    }
}

struct GridBlockMetaRevisionCloudService {
    #[allow(dead_code)]
    token: String,
}

impl RevisionCloudService for GridBlockMetaRevisionCloudService {
    #[tracing::instrument(level = "trace", skip(self))]
    fn fetch_object(&self, _user_id: &str, _object_id: &str) -> FutureResult<Vec<Revision>, FlowyError> {
        FutureResult::new(async move { Ok(vec![]) })
    }
}

struct GridBlockMetaPadBuilder();
impl RevisionObjectBuilder for GridBlockMetaPadBuilder {
    type Output = GridBlockMetaPad;

    fn build_object(object_id: &str, revisions: Vec<Revision>) -> FlowyResult<Self::Output> {
        let pad = GridBlockMetaPad::from_revisions(object_id, revisions)?;
        Ok(pad)
    }
}

struct GridBlockMetaRevisionCompactor();
impl RevisionCompactor for GridBlockMetaRevisionCompactor {
    fn bytes_from_revisions(&self, revisions: Vec<Revision>) -> FlowyResult<Bytes> {
        let delta = make_delta_from_revisions::<PlainTextAttributes>(revisions)?;
        Ok(delta.to_bytes())
    }
}