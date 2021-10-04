use std::borrow::Borrow;

use anyhow::{anyhow, Error};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::console;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};
use yew::format::{Bincode, Json, Nothing};
use yew::html::Scope;
use yew::services::fetch::{FetchTask, Request, Response};
use yew::services::FetchService;
use yew::{ChangeData, Component, Html, NodeRef, html};

use crate::la::{Matrix, Vec3f};
use crate::tga::Image;
use crate::wf::Wavefront;
use crate::{get_look_at, look_at, triangle, BasicShader, Shader};

pub enum Msg {
    Noop,
    Render(),
    ReRender(),
    Texture(Vec<u8>),
    Model(Vec<u8>),
    Normals(Vec<u8>),
    Upd(Vec3f),
}

pub struct Model {
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

        // println!("{:?}", lookat.mul(&lookat_i));
        let model = self.model.as_ref().unwrap();
        let texture = self.texture.as_ref().unwrap();
        let mut shader = BasicShader {
            light_dir: light_dir.normalize(),
            lookat_m: lookat,
            lookat_mi: lookat_i,
            model,
            mod_texture: texture,
            out_texture: &mut out_texture,
            norm_texture: &self.normals.as_ref().unwrap(),
            z_buffer: &mut z_buffer,
            varying_uv: Matrix::zeroed(3, 2),
            varying_xy: Matrix::zeroed(3, 3),
        };

        for f in 0..model.faces.len() {
            let mut vertices = [Vec3f::zeroed(), Vec3f::zeroed(), Vec3f::zeroed()];
            for v in 0..3 {
                vertices[v] = shader.vertex(f, v);
            }
            triangle(&vertices[0], &vertices[1], &vertices[2], &mut shader);
        }

        out_texture.apply_gamma(1.5);
        // out_texture.write_to_tga("african_head.tga").unwrap();
        // z_buffer.write_to_tga("zbuff.tga").unwrap();

        let canvas = self.node_ref.cast::<HtmlCanvasElement>().unwrap();
        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();
        let img = out_texture.get_raw_bytes();
        let id = ImageData::new_with_u8_clamped_array(Clamped(&img[..]), 512).unwrap();
        ctx.put_image_data(&id, 0.0, 0.0).unwrap();
    }

    fn ready(&self) -> bool {
        self.texture.is_some() && self.model.is_some() && self.normals.is_some()
    }

    fn load_binary(&mut self, url: String, dispatch: fn(Vec<u8>) -> Msg) {
        let get_request = Request::get(url)
            .body(Nothing)
            .expect("Could not build that request");
        let callback = self
            .link
            .callback(move |response: Response<Result<Vec<u8>, Error>>| {
                let data = response.into_body();
                let r = data.unwrap();
                dispatch(r)
                // Msg::Texture(r)
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
            task: Vec::new(),
            link: link,
            props: props,
            node_ref: NodeRef::default(),
            texture: None,
            model: None,
            normals: None,
            campos: Vec3f(0.5, 0.5, 1.0),
        }
    }

    fn rendered(&mut self, first_render: bool) {
        if first_render {
            self.load_binary("textr23.tga".to_owned(), |v| Msg::Texture(v));
            self.load_binary("nm.tga".to_owned(), |v| Msg::Normals(v));
            self.load_binary("african_head.obj".to_owned(), |v| Msg::Model(v));
        }
    }

    fn update(&mut self, msg: Self::Message) -> yew::ShouldRender {
        match msg {
            Msg::Noop => {
                false
            }
            Msg::Upd(v) => {
                self.campos = v;
                if self.ready() {
                    self.render();
                }
                true
                // self.campos
            }
            Msg::Render() => {
                self.load_binary("textr23.tga".to_owned(), |v| Msg::Texture(v));
                self.load_binary("nm.tga".to_owned(), |v| Msg::Normals(v));
                self.load_binary("african_head.obj".to_owned(), |v| Msg::Model(v));
                false
            }
            Msg::ReRender() => {
                if self.ready() {
                    self.render();
                }
                false
            }
            Msg::Texture(v) => {
                self.texture = Some(Image::from_raw_vec(v));
                if self.ready() {
                    self.render();
                }
                false
            }
            Msg::Normals(v) => {
                self.normals = Some(Image::from_raw_vec(v));
                if self.ready() {
                    self.render();
                }
                false
            }
            Msg::Model(v) => {
                self.model = Some(Wavefront::parse_string(String::from_utf8(v).unwrap()));
                if self.ready() {
                    self.render();
                }
                false
            }
        }
    }

    fn view(&self) -> Html {
        let Vec3f(x, y, z) = self.campos.clone();
        html! {
            <div style="display: flex;justify-content: center;align-items: center">
                <canvas ref={self.node_ref.clone()} width="512" height="512" />
                <div style="display: flex;align-items: flex-start;flex-direction: column;">
                    // <button onclick=self.link.callback(|_| Msg::ReRender())>{ "ReRender" }</button>
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
                </div>
            </div>
        }
    }
}

pub fn web() {
    yew::start_app::<Model>();
}
