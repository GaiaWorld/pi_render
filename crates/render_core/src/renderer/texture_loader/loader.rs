use crossbeam::queue::SegQueue;
use ktx::KtxInfo;
use pi_assets::{asset::{GarbageEmpty, Handle}, mgr::{AssetMgr, LoadResult}};
use pi_async_rt::prelude::AsyncRuntime;
use pi_hal::runtime::RENDER_RUNTIME;
use pi_hash::{XHashMap, XHashSet};
use crate::{renderer::texture::{ImageTextureFrame, KeyImageTextureFrame}, rhi::{device::RenderDevice, RenderQueue}};
use pi_share::Share;
use pi_assets::mgr::Receiver;
use pi_hal::image::DynamicImage;

use crate::renderer::{errors::{EError, ErrorRecord}, texture_loader::{environment_texture_loader::*, texture_atlas::*}};

pub type IDImageTextureLoad = u64;

#[derive(Clone, Copy)]
pub enum ETextureLoaderMode {
    D2,
    Env,
}

pub struct QueueInfo {
    pub id: IDImageTextureLoad,
    pub key: KeyImageTextureFrame,
    pub mode: ETextureLoaderMode,
}

#[derive(Default)]
pub struct StateTextureLoader {
    pub image_count: u32,
    pub image_success: u32,
    pub image_fail: u32,
    pub image_waiting: u32,
    pub texview_count: u32,
    pub texview_success: u32,
    pub texview_fail: u32,
    pub texview_waiting: u32,
}

#[derive(Clone)]
pub struct ImageTextureLoader {
    pub wait: Share<SegQueue<QueueInfo>>,
    pub success_load: Share<SegQueue<IDImageTextureLoad>>,
    pub fails: Share<SegQueue<IDImageTextureLoad>>,
    pub loading: XHashSet<KeyImageTextureFrame>,
    pub loading_image: Share<SegQueue<(KeyImageTextureFrame, DynamicImage, Receiver<ImageTextureFrame, GarbageEmpty>)>>,
    pub loading_data: Share<SegQueue<(KeyImageTextureFrame, Share<Vec<u8>>, Receiver<ImageTextureFrame, GarbageEmpty>)>>,
    pub fail_reason: XHashMap<KeyImageTextureFrame, EError>,
    pub fail_imgtex: Share<SegQueue<(KeyImageTextureFrame, EError)>>,
    pub success: XHashMap<IDImageTextureLoad, Handle<ImageTextureFrame>>,
    pub failrecord: XHashMap<IDImageTextureLoad, EError>,
    pub query_counter: IDImageTextureLoad,
}
impl Default for ImageTextureLoader {
    fn default() -> Self {
        Self {
            wait: Share::new(SegQueue::new()),
            success_load: Share::new(SegQueue::new()),
            loading: XHashSet::default(),
            loading_image: Share::new(SegQueue::new()),
            loading_data: Share::new(SegQueue::new()),
            fails: Share::new(SegQueue::new()),
            fail_reason: XHashMap::default(),
            fail_imgtex: Share::new(SegQueue::new()),
            success: XHashMap::default(),
            failrecord: XHashMap::default(),
            query_counter: 0,
        }
    }
}
impl ImageTextureLoader {
    pub fn size(&self) -> usize {
        self.wait.len() * 32
        + self.success_load.len() * 8
        + self.fails.len() * 8
        + self.fail_reason.len() * 20
        + self.fail_imgtex.len() * 20
        + self.success.len() * 8
        + self.failrecord.len() * 1
    }
    pub fn create_load(&mut self, key: KeyImageTextureFrame) -> IDImageTextureLoad {
        self.query_counter += 1;
        let id = self.query_counter;
        self.wait.push(QueueInfo { id, key, mode: ETextureLoaderMode::D2 });
        id
    }
    pub fn create_load_env(&mut self, key: KeyImageTextureFrame) -> IDImageTextureLoad {
        self.query_counter += 1;
        let id = self.query_counter;
        self.wait.push(QueueInfo { id, key, mode: ETextureLoaderMode::Env });
        id
    }
    ///
    /// 查询 Image 纹理状态, 
    /// 加载成功 返回资源引用
    /// 加载失败 返回 Err(true)
    /// 加载中 返回 Err(false)
    pub fn query_imgtex(&self, key: &KeyImageTextureFrame, asset: &AssetMgr<ImageTextureFrame>) -> Result<Handle<ImageTextureFrame>, bool> {
        if let Some(res) = asset.get(key) {
            Ok(res)
        } else {
            Err(self.fail_reason.contains_key(key))
        }
    }
    pub fn query_failed_reason(&mut self, id: IDImageTextureLoad) -> Option<EError> {
        if let Some(key) = self.failrecord.remove(&id) {
            Some(key)
        } else {
            None
        }
    }
    pub fn query_success(&mut self, id: IDImageTextureLoad) -> Option<Handle<ImageTextureFrame>> {
        self.success.remove(&id)
    }
}

pub fn image_texture_load_launch(
    mut loader: &mut ImageTextureLoader,
    image_assets_mgr: &Share<AssetMgr<ImageTextureFrame>>,
    queue: &RenderQueue,
    device: &RenderDevice,
    mut state: &mut StateTextureLoader,
) {
    let useage = wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC;
    let mut again = vec![];
    let mut item = loader.wait.pop();
    let mut idcounter = 0;
    while let Some(info) = item {
        idcounter = idcounter + 1;
        // if idcounter >= 1024 {
        //     log::error!("sys_image_texture_loaded");
        // }
        let id = info.id;
        let param = info.key.clone();
        let mode = info.mode;
        item = loader.wait.pop();

        if let Some(res) = image_assets_mgr.get(&param) {
            if id > 0 {
                loader.success_load.push(id);
                loader.success.insert(id, res);
            }
            continue;
        }
        let imageresult = AssetMgr::load(&image_assets_mgr, &param);
        match mode {
            ETextureLoaderMode::D2 => {
                match imageresult {
                    pi_assets::mgr::LoadResult::Ok(res) => {
                        if id > 0 {
                            loader.success_load.push(id);
                            loader.success.insert(id, res);
                        }
                    },
                    pi_assets::mgr::LoadResult::Wait(f) => {
                        if id > 0 { again.push(info); }
                        let (failquene, _device, _queue) = (loader.fail_imgtex.clone(), (device).clone(), (queue).clone());
                        RENDER_RUNTIME.spawn(async move {
                            match f.await {
                                Ok(_result) => { },
                                Err(_err) => failquene.push((param.clone(), ErrorRecord::ERROR_TEXTURE_CACHE_FAIL))
                            }
                        })
                        .unwrap();
                    },
                    LoadResult::Receiver(recv) => {
                        if let Some(err) = loader.fail_reason.get(&param) {
                            if id > 0 {
                                loader.fails.push(id);
                                loader.failrecord.insert(id, *err);
                                state.image_fail += 1;
                            }
                        } else {
                            match &param.file {
                                false => loader.fail_imgtex.push((param, ErrorRecord::ERROR_TEXTURE_CANT_LOAD_FROM_DATA)),
                                true => {
                                    if id > 0 { again.push(info); }
                                    let (failquene, device, queue) = (loader.fail_imgtex.clone(), (device).clone(), (queue).clone());
                                    let (loading_img, loading_data) = (loader.loading_image.clone(), loader.loading_data.clone());

                                    if param.cancombine {
                                        if loader.loading.contains(&param) == false {
                                            loader.loading.insert(param.clone());
                                            let param = param.clone();
                                            RENDER_RUNTIME.spawn(async move {
                                                if param.compressed {
                                                    match pi_hal::file::load_from_url(&param.url).await {
                                                        Ok(data) => loading_data.push((param, data, recv)),
                                                        Err(_) => failquene.push((param, ErrorRecord::ERROR_TEXTURE_LOAD_FAIL)),
                                                    }
                                                } else {
                                                    match pi_hal::image::load_from_url(&param.url).await {
                                                        Ok(img) => loading_img.push((param, img, recv)),
                                                        Err(_) => failquene.push((param, ErrorRecord::ERROR_TEXTURE_LOAD_FAIL)),
                                                    }
                                                }
                                            })
                                            .unwrap();
                                        }
                                    } else {
                                        let param = param.clone();
                                        RENDER_RUNTIME.spawn(async move {
                                            let haldesc = pi_hal::texture::ImageTextureDesc { url: param.url.clone(), srgb: false, useage, };
                                            match pi_hal::image_texture_load::load_from_url(&haldesc, &device, &queue).await {
                                                Ok(data) => {
                                                    match recv.receive(param.clone(), Ok(ImageTextureFrame::new(data))).await {
                                                        Ok(_result) => {},
                                                        Err(_) => failquene.push((param, ErrorRecord::ERROR_TEXTURE_CACHE_FAIL))
                                                    }
                                                },
                                                Err(_) => {
                                                    failquene.push((param, ErrorRecord::ERROR_TEXTURE_LOAD_FAIL));
                                                },
                                            };
                                        })
                                        .unwrap();
                                    }
                                },
                            }
                        }
                    }
                }
            },
            ETextureLoaderMode::Env => {
                match imageresult {
                    pi_assets::mgr::LoadResult::Ok(res) => {
                        if id > 0 {
                            loader.success_load.push(id);
                            loader.success.insert(id, res);
                        }
                    },
                    _ => {
                        if id > 0 { again.push(info); }
                        if param.file {
                            loader.fail_imgtex.push((param.clone(), ErrorRecord::ERROR_TEXTURE_LOAD_FAIL));
                        } else {
                            let (failquene, device, queue) = (loader.fail_imgtex.clone(), (device).clone(), (queue).clone());
                            let param = param.clone();
                            RENDER_RUNTIME.spawn(async move {
                                match EnvironmentTextureTools::async_load(param.clone(), device, queue, imageresult).await {
                                    Ok(_) => {},
                                    Err(_) => failquene.push((param, ErrorRecord::ERROR_TEXTURE_LOAD_FAIL)),
                                }
                            })
                            .unwrap();
                        }
                    }
                }
            },
        }
    }

    again.drain(..).for_each(|item| { loader.wait.push(item); });
}


pub fn image_texture_loaded(
    mut loader: &mut ImageTextureLoader,
    mut state: &mut StateTextureLoader,
    mut combinemgr: &mut TextureCombineAtlas2DMgr,
    queue: &RenderQueue,
    device: &RenderDevice,
) {
    let mut idcounter = 0;
    while let Some((keyimage, data, receiver)) = loader.loading_image.pop() {
        idcounter = idcounter + 1;
        if idcounter >= 1024 {
            log::error!("sys_image_texture_loaded");
        }
        loader.loading.remove(&keyimage);
        let failquene = loader.fail_imgtex.clone();
        if let Some(texture) = combinemgr.combine_image(&keyimage, &data, &device, &queue) {
            RENDER_RUNTIME.spawn(async move {
                match receiver.receive(keyimage.clone(), Ok(texture)).await {
                    Ok(_) => {},
                    Err(_) => {
                        failquene.push((keyimage, ErrorRecord::ERROR_TEXTURE_COMBINE_FAIL));
                    },
                }
            })
            .unwrap();
        } else if let Some(texture) = ImageTextureFrame::create_image(&device, &queue, &keyimage.url, wgpu::TextureViewDimension::D2, data) {
            RENDER_RUNTIME.spawn(async move {
                match receiver.receive(keyimage.clone(), Ok(ImageTextureFrame::new(texture))).await {
                    Ok(_) => {},
                    Err(_) => {
                        failquene.push((keyimage, ErrorRecord::ERROR_TEXTURE_CREATE_FAIL));
                    },
                }
            })
            .unwrap();
        } else {
            failquene.push((keyimage, ErrorRecord::ERROR_TEXTURE_LOAD_FAIL));
        }
    }
    while let Some((keyimage, data, receiver)) = loader.loading_data.pop() {
        loader.loading.remove(&keyimage);
        let failquene = loader.fail_imgtex.clone();
        if let Some(texture) = combinemgr.combine_ktx(&keyimage, &data, &device, &queue) {
            RENDER_RUNTIME.spawn(async move {
                match receiver.receive(keyimage.clone(), Ok(texture)).await {
                    Ok(_) => {},
                    Err(_) => {
                        failquene.push((keyimage, ErrorRecord::ERROR_TEXTURE_COMBINE_FAIL));
                    },
                }
            })
            .unwrap();
        } else {
            let ktx = ktx::Ktx::new(data.as_slice());
            if let Some(format) = compressed_texture_format(ktx.gl_internal_format()) {
                if let Some(texture) = ImageTextureFrame::create_ktx(&device, &queue, &keyimage.url, wgpu::TextureViewDimension::D2, format, &ktx) {
                    RENDER_RUNTIME.spawn(async move {
                        match receiver.receive(keyimage.clone(), Ok(ImageTextureFrame::new(texture))).await {
                            Ok(_) => {},
                            Err(_) => {
                                failquene.push((keyimage, ErrorRecord::ERROR_TEXTURE_CREATE_FAIL));
                            },
                        }
                    })
                    .unwrap();
                } else {
                    loader.fail_imgtex.push((keyimage, ErrorRecord::ERROR_TEXTURE_CREATE_FAIL));
                }
            } else {
                loader.fail_imgtex.push((keyimage, ErrorRecord::ERROR_TEXTURE_FROM_KTX_FAIL));
            }
        }
    }
    let mut item = loader.fail_imgtex .pop();
    while let Some((param, error)) = item {
        item = loader.fail_imgtex.pop();
        loader.fail_reason.insert(param, error);
        state.image_fail += 1;
    }
}
