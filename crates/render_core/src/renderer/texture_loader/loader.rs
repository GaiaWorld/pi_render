use std::sync::Arc;

use crossbeam::queue::SegQueue;
use ktx::KtxInfo;
use pi_assets::{asset::{GarbageEmpty, Handle}, mgr::{AssetMgr, LoadResult}};
use pi_async_rt::prelude::AsyncRuntime;
use pi_hal::runtime::RENDER_RUNTIME;
use wgpu::TextureUsages;
use crate::{renderer::texture::{ImageTextureFrame, KeyImageTextureFrame}, rhi::{device::RenderDevice, RenderQueue}};
use pi_share::Share;
use pi_assets::mgr::Receiver;
use pi_hal::image::DynamicImage;

use crate::renderer::{errors::{EError, ErrorRecord}, texture_loader::{texture_atlas::*}};

pub type IDImageTextureLoad = u64;

pub trait TTextureLoaderImpl: Clone {
    fn async_load(
        param: KeyImageTextureFrame,
        useage: TextureUsages,
        device: &RenderDevice, queue: &RenderQueue,
        image_assets_mgr: &Share<AssetMgr<ImageTextureFrame>>,
        success: Share<SegQueue<Self>>,
        tempdata: Share<SegQueue<(KeyImageTextureFrame, Arc<Vec<u8>>, Receiver<ImageTextureFrame, GarbageEmpty>)>>,
        tempimage: Share<SegQueue<(KeyImageTextureFrame, DynamicImage, Receiver<ImageTextureFrame, GarbageEmpty>)>>,
        failquene: Share<SegQueue<(KeyImageTextureFrame, EError)>>,
        combinemgr: &mut TextureCombineAtlas2DMgr,
    );
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
pub struct ImageTextureLoader<K: Send + 'static> {
    pub loading_image: Share<SegQueue<(K, KeyImageTextureFrame, DynamicImage, Receiver<ImageTextureFrame, GarbageEmpty>)>>,
    pub loading_data: Share<SegQueue<(K, KeyImageTextureFrame, Share<Vec<u8>>, Receiver<ImageTextureFrame, GarbageEmpty>)>>,
    pub success: Share<SegQueue<(K, KeyImageTextureFrame, Handle<ImageTextureFrame>)>>,
    pub failquene: Share<SegQueue<(K, KeyImageTextureFrame, EError)>>,
}
impl<K: Send + 'static> Default for ImageTextureLoader<K> {
    fn default() -> Self {
        Self {
            loading_image: Share::new(SegQueue::new()),
            loading_data: Share::new(SegQueue::new()),
            success: Share::new(SegQueue::new()),
            failquene: Share::new(SegQueue::new()),
        }
    }
}
impl<K: Send + 'static> ImageTextureLoader<K> {
    pub fn size(&self) -> usize {
        self.failquene.len() * 8
        + self.loading_image.len() * 20
        + self.loading_data.len() * 20
        + self.success.len() * 8
    }
    pub fn async_load(
        loadtaskkey: K,
        param: KeyImageTextureFrame,
        useage: TextureUsages,
        device: &RenderDevice, queue: &RenderQueue,
        image_assets_mgr: &Share<AssetMgr<ImageTextureFrame>>,
        success: &Share<SegQueue<(K, KeyImageTextureFrame, Handle<ImageTextureFrame>)>>,
        failquene: &Share<SegQueue<(K, KeyImageTextureFrame, EError)>>,
        tempdata: &Share<SegQueue<(K, KeyImageTextureFrame, Arc<Vec<u8>>, Receiver<ImageTextureFrame, GarbageEmpty>)>>,
        tempimage: &Share<SegQueue<(K, KeyImageTextureFrame, DynamicImage, Receiver<ImageTextureFrame, GarbageEmpty>)>>,
    ) -> Option<Handle<ImageTextureFrame>> {
        let imageresult = AssetMgr::load(&image_assets_mgr, &param);
        match imageresult {
            LoadResult::Ok(data) => return Some(data),
            LoadResult::Wait(pin) => {
                let success = success.clone();
                let failquene = failquene.clone();
                RENDER_RUNTIME.spawn(async move {
                    match pin.await {
                        Ok(data) => success.push((loadtaskkey, param, data)),
                        Err(_) => failquene.push((loadtaskkey, param, ErrorRecord::ERROR_TEXTURE_CANT_LOAD_FROM_DATA)),
                    }
                })
                .unwrap();
            },
            LoadResult::Receiver(receiver) => {
                match &param.file {
                    false => failquene.push((loadtaskkey, param, ErrorRecord::ERROR_TEXTURE_CANT_LOAD_FROM_DATA)),
                    true => {
                        let (device, queue) = ((device).clone(), (queue).clone());
                        // 允许合并, 先加载文件,再尝试进行合并
                        if param.cancombine {
                            let param = param.clone();
                            let tempdata = tempdata.clone();
                            let tempimage = tempimage.clone();
                            let failquene = failquene.clone();
                            RENDER_RUNTIME.spawn(async move {
                                if param.compressed {
                                    match pi_hal::file::load_from_url(&param.url).await {
                                        Ok(data) => tempdata.push((loadtaskkey, param, data, receiver)),
                                        Err(_) => failquene.push((loadtaskkey, param, ErrorRecord::ERROR_TEXTURE_LOAD_FAIL)),
                                    }
                                } else {
                                    match pi_hal::image::load_from_url(&param.url).await {
                                        Ok(img) => tempimage.push((loadtaskkey, param, img, receiver)),
                                        Err(_) => failquene.push((loadtaskkey, param, ErrorRecord::ERROR_TEXTURE_LOAD_FAIL)),
                                    }
                                }
                            })
                            .unwrap();
                        
                        // 不允许合并, 直接先加载为纹理
                        } else {
                            let param = param.clone();
                            let success = success.clone();
                            let failquene = failquene.clone();
                            RENDER_RUNTIME.spawn(async move {
                                let haldesc = pi_hal::texture::ImageTextureDesc { url: param.url.clone(), srgb: false, useage, };
                                match pi_hal::image_texture_load::load_from_url(&haldesc, &device, &queue).await {
                                    Ok(data) => {
                                        match receiver.receive(param.clone(), Ok(ImageTextureFrame::new(data))).await {
                                            Ok(_result) => success.push((loadtaskkey, param, _result)),
                                            Err(_) => failquene.push((loadtaskkey, param, ErrorRecord::ERROR_TEXTURE_CACHE_FAIL))
                                        }
                                    },
                                    Err(_) => failquene.push((loadtaskkey, param, ErrorRecord::ERROR_TEXTURE_LOAD_FAIL)),
                                };
                            })
                            .unwrap();
                        }
                    },
                }
            },
        }
        return None;
    }
    pub fn check_combine(
        device: &RenderDevice, queue: &RenderQueue,
        combinemgr: &mut TextureCombineAtlas2DMgr,
        success: &Share<SegQueue<(K, KeyImageTextureFrame, Handle<ImageTextureFrame>)>>,
        failquene: &Share<SegQueue<(K, KeyImageTextureFrame, EError)>>,
        tempdata: &Share<SegQueue<(K, KeyImageTextureFrame, Arc<Vec<u8>>, Receiver<ImageTextureFrame, GarbageEmpty>)>>,
        tempimage: &Share<SegQueue<(K, KeyImageTextureFrame, DynamicImage, Receiver<ImageTextureFrame, GarbageEmpty>)>>,
    ) {
        // 压缩纹理合并
        while let Some((key, keyimage, data, receiver)) = tempdata.pop() {
            if let Some(texture) = combinemgr.combine_ktx(&keyimage, &data, &device, &queue) {
                let success = success.clone();
                let failquene = failquene.clone();
                RENDER_RUNTIME.spawn(async move {
                    match receiver.receive(keyimage.clone(), Ok(texture)).await {
                        Ok(data) => success.push((key, keyimage, data)),
                        Err(_) => failquene.push((key, keyimage, ErrorRecord::ERROR_TEXTURE_COMBINE_FAIL)),
                    }
                })
                .unwrap();
            // 合并失败创建为独立纹理
            } else {
                let ktx = ktx::Ktx::new(data.as_slice());
                if let Some(format) = compressed_texture_format(ktx.gl_internal_format()) {
                    if let Some(texture) = ImageTextureFrame::create_ktx(&device, &queue, &keyimage.url, wgpu::TextureViewDimension::D2, format, &ktx) {
                        let success = success.clone();
                        let failquene = failquene.clone();
                        RENDER_RUNTIME.spawn(async move {
                            match receiver.receive(keyimage.clone(), Ok(ImageTextureFrame::new(texture))).await {
                                Ok(data) => success.push((key, keyimage, data)),
                                Err(_) => failquene.push((key, keyimage, ErrorRecord::ERROR_TEXTURE_CREATE_FAIL)),
                            }
                        })
                        .unwrap();
                    } else { failquene.push((key, keyimage, ErrorRecord::ERROR_TEXTURE_CREATE_FAIL)); }
                } else { failquene.push((key, keyimage, ErrorRecord::ERROR_TEXTURE_FROM_KTX_FAIL)); }
            }
        }
        // Image 纹理合并
        while let Some((key, keyimage, data, receiver)) = tempimage.pop() {
            if let Some(texture) = combinemgr.combine_image(&keyimage, &data, &device, &queue) {
                let success = success.clone();
                let failquene = failquene.clone();
                RENDER_RUNTIME.spawn(async move {
                    match receiver.receive(keyimage.clone(), Ok(texture)).await {
                        Ok(data) => { success.push((key, keyimage, data)) },
                        Err(_) => failquene.push((key, keyimage, ErrorRecord::ERROR_TEXTURE_COMBINE_FAIL)),
                    }
                })
                .unwrap();
            // 合并失败创建为独立纹理
            } else if let Some(texture) = ImageTextureFrame::create_image(&device, &queue, &keyimage.url, wgpu::TextureViewDimension::D2, data) {
                let success = success.clone();
                let failquene = failquene.clone();
                RENDER_RUNTIME.spawn(async move {
                    match receiver.receive(keyimage.clone(), Ok(ImageTextureFrame::new(texture))).await {
                        Ok(data) => { success.push((key, keyimage, data)) },
                        Err(_) => failquene.push((key, keyimage, ErrorRecord::ERROR_TEXTURE_CREATE_FAIL)),
                    }
                })
                .unwrap();
            } else { failquene.push((key, keyimage, ErrorRecord::ERROR_TEXTURE_LOAD_FAIL)); }
        }
    }
}
