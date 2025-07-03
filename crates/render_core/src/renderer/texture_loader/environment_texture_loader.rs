
use pi_assets::{asset::{Garbageer, Handle}, mgr::{Receiver, LoadResult}};
use pi_futures::BoxFuture;
use crate::{renderer::texture::{ErrorImageTexture, ImageTextureFrame, KeyImageTextureFrame}, rhi::{device::RenderDevice, internal::bytemuck, RenderQueue}};
use serde::Deserialize;

use crate::renderer::errors::{EError, ErrorRecord};

#[derive(Deserialize)]
#[allow(dead_code)]
struct Irradiance {
    polynomials: Option<bool>,
    x: [f64; 3],
    y: [f64; 3],
    z: [f64; 3],
    xx: [f64; 3],
    yy: [f64; 3],
    zz: [f64; 3],
    yz: [f64; 3],
    zx: [f64; 3],
    xy: [f64; 3],
}

#[derive(Deserialize)]
struct Mipmap {
    length: u64,
    position: u64,
}

#[derive(Deserialize)]
#[allow(non_snake_case)]
struct Specular {
    mipmaps: Vec<Mipmap>,
    lodGenerationScale: Option<f64>,
    specularDataPosition: Option<u64>
}

#[derive(Deserialize)]
struct EnvironmentInfo {
    #[allow(dead_code)]
    version: u64,
    width: u64,
    irradiance: Irradiance,
    specular: Specular,
}

pub struct EnvironmentTextureTools {
    pub width: u32,
    pub mipmaplevels: u32,
    pub infodata: Vec<u8>,
    specular: Specular,
}
impl EnvironmentTextureTools {
    pub const IRRADIANCE_SIZE: usize = 9 * (3 + 1) * 4;
    const MAGIC_BYTES: [u8;8] = [0x86, 0x16, 0x87, 0x96, 0xf6, 0xd6, 0x96, 0x36];
    ///
    /// * BabylonJS Env Spherical
    /// ```ts
    ///   this._activeEffect.setFloat3("vSphericalX", polynomials.x.x, polynomials.x.y, polynomials.x.z);
    ///   this._activeEffect.setFloat3("vSphericalY", polynomials.y.x, polynomials.y.y, polynomials.y.z);
    ///   this._activeEffect.setFloat3("vSphericalZ", polynomials.z.x, polynomials.z.y, polynomials.z.z);
    ///   this._activeEffect.setFloat3("vSphericalXX_ZZ", polynomials.xx.x - polynomials.zz.x, polynomials.xx.y - polynomials.zz.y, polynomials.xx.z - polynomials.zz.z);
    ///   this._activeEffect.setFloat3("vSphericalYY_ZZ", polynomials.yy.x - polynomials.zz.x, polynomials.yy.y - polynomials.zz.y, polynomials.yy.z - polynomials.zz.z);
    ///   this._activeEffect.setFloat3("vSphericalZZ", polynomials.zz.x, polynomials.zz.y, polynomials.zz.z);
    ///   this._activeEffect.setFloat3("vSphericalXY", polynomials.xy.x, polynomials.xy.y, polynomials.xy.z);
    ///   this._activeEffect.setFloat3("vSphericalYZ", polynomials.yz.x, polynomials.yz.y, polynomials.yz.z);
    ///   this._activeEffect.setFloat3("vSphericalZX", polynomials.zx.x, polynomials.zx.y, polynomials.zx.z);
    /// ```
    pub fn get_env_info(data: &[u8]) -> Result<Self, EError> {
        let len = Self::MAGIC_BYTES.len();
        let mut pos = 0;
        for i in 0..len {
            if data[pos] != Self::MAGIC_BYTES[i] {
                return Err(ErrorRecord::ERROR_ENVIRONMENT_INFO_MAGICNUMBER);
            }
            pos += 1;
        }

        let mut manifest_string = String::from("");
        let mut check_char_code: u8;
        loop {
            check_char_code = data[pos];
            pos += 1;
            if let Some(char) = char::from_u32(check_char_code as u32) {
                if check_char_code == 0x00 {
                    break;
                }
                manifest_string += char.to_string().as_str();
            } else {
                break;
            };
        };

        match serde_json::from_str::<EnvironmentInfo>(&manifest_string) {
            Ok(mut info) => {
                let mut infodata = Vec::with_capacity(9 * 4);

                if info.specular.lodGenerationScale.is_none() {
                    info.specular.lodGenerationScale = Some(0.8);
                }
                
                let zz = &info.irradiance.zz;
                let mut i = 0;

                info.irradiance.x .iter().for_each(|v| { infodata.push(*v as f32); }); infodata.push(info.width as f32);
                info.irradiance.y .iter().for_each(|v| { infodata.push(*v as f32); }); infodata.push(info.specular.lodGenerationScale.unwrap() as f32);
                info.irradiance.z .iter().for_each(|v| { infodata.push(*v as f32); }); infodata.push(0.);
                info.irradiance.xx.iter().for_each(|v| { infodata.push(*v as f32 - zz[i] as f32); i += 1; }); infodata.push(0.); i = 0;
                info.irradiance.yy.iter().for_each(|v| { infodata.push(*v as f32 - zz[i] as f32); i += 1; }); infodata.push(0.); // i = 0;
                info.irradiance.zz.iter().for_each(|v| { infodata.push(*v as f32); }); infodata.push(0.);
                info.irradiance.xy.iter().for_each(|v| { infodata.push(*v as f32); }); infodata.push(0.);
                info.irradiance.yz.iter().for_each(|v| { infodata.push(*v as f32); }); infodata.push(0.);
                info.irradiance.zx.iter().for_each(|v| { infodata.push(*v as f32); }); infodata.push(0.);
                
                let mipmaplevels = info.specular.mipmaps.len() / 6;

                let v: &[u8] = bytemuck::cast_slice(&infodata);

                if info.specular.specularDataPosition.is_none() {
                    info.specular.specularDataPosition = Some(pos as u64);
                }

                Ok(Self {
                    width: info.width as u32,
                    mipmaplevels: mipmaplevels as u32,
                    infodata: v.to_vec(),
                    specular: info.specular,
                })
            },
            Err(_err) => { Err(ErrorRecord::ERROR_ENVIRONMENT_INFO_PARSE) },
        }
    }

    pub fn async_load<'a, G: Garbageer<ImageTextureFrame>>(desc: KeyImageTextureFrame, device: RenderDevice, queue: RenderQueue, result: LoadResult<'a, ImageTextureFrame, G>) -> BoxFuture<'a, Result<Handle<ImageTextureFrame>, ErrorImageTexture>> {
        Box::pin(async move { 
            match result {
                LoadResult::Ok(r) => Ok(r),
                LoadResult::Wait(f) => match f.await {
                    Ok(result) => Ok(result),
                    Err(_e) => { Err(ErrorImageTexture::LoadFail)},
                },
                LoadResult::Receiver(recv) => {
                    match pi_hal::file::load_from_url( &desc.url ).await {
                        Ok(data) => create_environment_texture_from_file(&data, desc, device, queue, recv).await,
                        Err(_e) => { Err(ErrorImageTexture::LoadFail) },
                    }
                }
            }
        })
    }

}


pub async fn create_environment_texture_from_file<G: Garbageer<ImageTextureFrame>>(
    data: &Vec<u8>,
    desc: KeyImageTextureFrame,
    device: RenderDevice, queue: RenderQueue,
    recv: Receiver<ImageTextureFrame, G>
) -> Result<Handle<ImageTextureFrame>, ErrorImageTexture> {
    
    // log::error!("Analy ");

    match EnvironmentTextureTools::get_env_info(data) {
        Ok(info) => {
            let width = info.width;
            let height = info.width;
            let format = wgpu::TextureFormat::Rgba8Unorm;
            let dimension = wgpu::TextureDimension::D2;
            let is_opacity = true;

            let pre_pixel_size = 4;

            let texture_extent = wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 6,
            };
            let texture = (**device).create_texture(&wgpu::TextureDescriptor {
                label: Some(desc.url.as_str()),
                size: texture_extent,
                mip_level_count: info.mipmaplevels,
                sample_count: 1,
                dimension,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let mut copytarget = texture.as_image_copy();
            
            // desc.queue.copy_external_image_to_texture
            let mut twidth = width;
            for mip_level in 0..info.mipmaplevels {
                // let mut buffer = Vec::with_capacity((width * width * 4) as usize);
                match desc.file {
                    true => {
                        // copytarget.mip_level = mip_level;
                        // for z in 0..6 {
                        //     let mipmap = info.specular.mipmaps.get((mip_level * 6 + z) as usize).unwrap();
                        //     let start = (info.specular.specularDataPosition.unwrap() + mipmap.position) as usize;
                        //     let end = start + mipmap.length as usize;
                        //     // log::error!("End : {:?}", (data[end - 1], mipmap.length, mipmap.position, start, end));
                        //     match image::load_from_memory(&data[start..end]) {
                        //         Ok(imagedata) => { buffer.append(&mut imagedata.into_bytes()); },
                        //         Err(_err) => { log::error!("{:?}", _err); return Err(ErrorImageTexture::LoadFail); },
                        //     }
                        // }
                        // let texture_extent = wgpu::Extent3d { width: twidth, height: twidth, depth_or_array_layers: 6, };
                        // desc.queue.write_texture( copytarget, &buffer, wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(twidth * pre_pixel_size), rows_per_image: Some(twidth) }, texture_extent );
                        // buffer.clear();
                    },
                    false => {
                        copytarget.mip_level = mip_level;
                        let mut start = info.specular.specularDataPosition.unwrap() as usize;
                        let mut end = 0;
                        for z in 0..6 {
                            let mipmap = info.specular.mipmaps.get((mip_level * 6 + z) as usize).unwrap();
                            if z == 0 { start += mipmap.position as usize; }
                            end += mipmap.length as usize;
                        }
                        end += start;
                        let buffer = &data[start..end];
                        let texture_extent = wgpu::Extent3d { width: twidth, height: twidth, depth_or_array_layers: 6, };
                        queue.write_texture( copytarget, buffer, wgpu::ImageDataLayout { offset: 0, bytes_per_row: Some(twidth * pre_pixel_size), rows_per_image: Some(twidth) }, texture_extent );
                    },
                }

                twidth /= 2;
            }
            // log::error!("Success ");

            let dimension = wgpu::TextureViewDimension::Cube;
            let haltex = pi_hal::texture::ImageTexture {
                width, height, size: data.len() as usize, texture, format, view_dimension: dimension, is_opacity
            };
            let mut texture = ImageTextureFrame::new(haltex);
            texture.extend = info.infodata;
            match recv.receive(desc, Ok(texture)).await {
                Ok(result) => Ok(result),
                Err(_) => Err(ErrorImageTexture::CreateError),
            }
        },
        Err(_) => Err(ErrorImageTexture::LoadFail),
    }
}