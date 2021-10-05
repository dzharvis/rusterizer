use anyhow::{Error};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::format::{Nothing};
use yew::services::fetch::{FetchTask, Request, Response, Uri};
use yew::services::FetchService;
use yew::{html, Component, Html, NodeRef};

use crate::la::{Matrix, Vec3f};
use crate::tga::Image;
use crate::model::Wavefront;
use crate::{get_look_at, look_at, triangle, BasicShader, Shader, ShaderConf};

pub enum Msg {
    Texture(Vec<u8>),
    Model(Vec<u8>),
    Normals(Vec<u8>),
    Upd(Vec3f),
    Diff,
    Spec,
    Txt,
    Zbuff,
    Norm,
}

pub struct Model {
    conf: ShaderConf,
    zbuff: bool,
    node_ref: NodeRef,
    props: (),
    link: yew::ComponentLink<Self>,
    task: Vec<Option<FetchTask>>,
    texture: Option<Image>,
    model: Option<Wavefront>,
    normals: Option<Image>,
    campos: Vec3f,
}

impl Model {
    fn render(&self) {
        let width: i32 = 512;
        let height: i32 = 512;
        let mut out_texture = Image::new(width, height);
        let mut z_buffer = Image::new(width, height);

        let campos = &self.campos;
        let lookat = get_look_at(&campos);
        let lookat_i = lookat.inverse().transpose();
        let light_dir: Vec3f = look_at(&lookat, &Vec3f(01.0, -0.0, 0.5).normalize());

        let model = self.model.as_ref().unwrap();
        let texture = self.texture.as_ref().unwrap();
        let mut shader = BasicShader {
            conf: self.conf.clone(),
            n: None,
            light_dir: light_dir.normalize(),
            lookat_m: lookat,
            lookat_mi: lookat_i,
            model,
            mod_texture: texture,
            out_texture: &mut out_texture,
            norm_texture: &self.normals.as_ref().unwrap(),
            z_buffer: &mut z_buffer,
            varying_uv: Matrix::zeroed(),
            varying_xy: Matrix::zeroed(),
        };

        for f in 0..model.faces.len() {
            let mut vertices = [Vec3f::zeroed(), Vec3f::zeroed(), Vec3f::zeroed()];
            for v in 0..3 {
                vertices[v] = shader.vertex(f, v);
            }
            triangle(&vertices[0], &vertices[1], &vertices[2], &mut shader);
        }

        out_texture.apply_gamma(1.5);

        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        let img = if self.zbuff { z_buffer } else { out_texture }.get_raw_bytes();
        let id = ImageData::new_with_u8_clamped_array(Clamped(&img[..]), 512).unwrap();
        ctx.put_image_data(&id, 0.0, 0.0).unwrap();
    }

    fn ready(&self) -> bool {
        self.texture.is_some() && self.model.is_some() && self.normals.is_some()
    }

    fn load_binary(&mut self, url: String, dispatch: fn(Vec<u8>) -> Msg) {
        let get_request = Request::get(Uri::builder().path_and_query(url).build().unwrap())
            .body(Nothing)
            .expect("Could not build that request");
        let callback = self
            .link
            .callback(move |response: Response<Result<Vec<u8>, Error>>| {
                let data = response.into_body().unwrap();
                dispatch(data)
            });
        let task =
            FetchService::fetch_binary(get_request, callback).expect("failed to start request");
        self.task.push(Some(task));
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn change(&mut self, _props: Self::Properties) -> yew::ShouldRender {
        if self.props != _props {
            self.props = _props;
            true
        } else {
            false
        }
    }

    fn create(props: Self::Properties, link: yew::ComponentLink<Self>) -> Self {
        Self {
            zbuff: false,
            conf: ShaderConf::new(),
            task: Vec::new(),
            link,
            props,
            node_ref: NodeRef::default(),
            texture: None,
            model: None,
            normals: None,
            campos: Vec3f(0.5, 0.5, 1.0),
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.load_binary("./african_head/texture.tga".to_owned(), |v| Msg::Texture(v));
            self.load_binary("./african_head/normals.tga".to_owned(), |v| Msg::Normals(v));
            self.load_binary("./african_head/model.obj".to_owned(), |v| Msg::Model(v));
        }
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            Msg::Zbuff => {
                self.zbuff = !self.zbuff;
                if self.ready() {
                    self.render();
                }
                true
            }
            Msg::Norm => {
                self.conf = ShaderConf {
                    normals: !self.conf.normals,
                    ..self.conf
                };
                if self.ready() {
                    self.render();
                }
                true
            }
            Msg::Diff => {
                self.conf = ShaderConf {
                    diff_light: !self.conf.diff_light,
                    ..self.conf
                };
                if self.ready() {
                    self.render();
                }
                false
            }
            Msg::Spec => {
                self.conf = ShaderConf {
                    spec_light: !self.conf.spec_light,
                    ..self.conf
                };
                if self.ready() {
                    self.render();
                }
                false
            }
            Msg::Txt => {
                self.conf = ShaderConf {
                    texture: !self.conf.texture,
                    ..self.conf
                };
                if self.ready() {
                    self.render();
                }
                false
            }
            Msg::Upd(v) => {
                self.campos = v;
                if self.ready() {
                    self.render();
                }
                true
            }
            Msg::Texture(v) => {
                self.texture = Some(Image::from_raw_vec(v));
                if self.ready() {
                    self.render();
                }
                true
            }
            Msg::Normals(v) => {
                self.normals = Some(Image::from_raw_vec(v));
                if self.ready() {
                    self.render();
                }
                true
            }
            Msg::Model(v) => {
                self.model = Some(Wavefront::parse_string(String::from_utf8(v).unwrap()));
                if self.ready() {
                    self.render();
                }
                true
            }
        }
    }

    fn view(&self) -> Html {
        let Vec3f(x, y, z) = self.campos.clone();
        html! {
            <div style="display: flex;justify-content: center;align-items: center">
                <canvas ref={self.node_ref.clone()} width="512" height="512" />
                <div style="display: flex;align-items: flex-start;flex-direction: column;">
                    { if self.ready() { html! {
                        <>
                            <div style="display: flex;align-items: flex-start">
                                <button onclick=self.link.callback(move |_| Msg::Upd(Vec3f(x+0.1, y, z)))>{ "+" }</button>
                                { "x: " }{ format!("{:.2}", x) }
                                <button onclick=self.link.callback(move |_| Msg::Upd(Vec3f(x-0.1, y, z)))>{ "-" }</button>
                            </div>
                            <div style="display: flex;align-items: flex-start">
                                <button onclick=self.link.callback(move |_| Msg::Upd(Vec3f(x, y+0.1, z)))>{ "+" }</button>
                                { "y: " }{ format!("{:.2}", y) }
                                <button onclick=self.link.callback(move |_| Msg::Upd(Vec3f(x, y-0.1, z)))>{ "-" }</button>
                            </div>
                            <div style="display: flex;align-items: flex-start">
                                <button onclick=self.link.callback(move |_| Msg::Upd(Vec3f(x, y, z+0.1)))>{ "+" }</button>
                                { "z: " }{ format!("{:.2}", z) }
                                <button onclick=self.link.callback(move |_| Msg::Upd(Vec3f(x, y, z-0.1)))>{ "-" }</button>
                            </div>
                            <button disabled={ self.zbuff } onclick=self.link.callback(move |_| Msg::Diff)>{ "Diffuse light" }</button>
                            <button disabled={ self.zbuff } onclick=self.link.callback(move |_| Msg::Spec)>{ "Specular light" }</button>
                            <button disabled={ self.zbuff } onclick=self.link.callback(move |_| Msg::Txt)>{ "Texture" }</button>
                            <button disabled={ self.zbuff } onclick=self.link.callback(move |_| Msg::Norm)>{ "Normal map" }</button>
                            <button onclick=self.link.callback(move |_| Msg::Zbuff)>{ "Z Buffer" }</button>
                        </>
                    } } else { html! { "Loading model.." } } }
                </div>
            </div>
        }
    }
}

pub fn web() {
    yew::start_app::<Model>();
}
