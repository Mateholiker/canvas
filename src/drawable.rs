use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use eframe::egui::{Rect, Response as EGuiResponse};

use crate::{CanvasHandle, Position};

pub trait Drawable {
    type DrawData;

    fn draw(&mut self, handle: &mut CanvasHandle, draw_data: &Self::DrawData);

    fn get_cutout(&mut self, draw_data: &Self::DrawData) -> Rect;

    #[allow(unused_variables)]
    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {}
}

impl<T, D> Drawable for &mut T
where
    T: Drawable<DrawData = D>,
{
    type DrawData = D;

    fn draw(&mut self, handle: &mut CanvasHandle, draw_data: &Self::DrawData) {
        (*self).draw(handle, draw_data);
    }

    fn get_cutout(&mut self, draw_data: &Self::DrawData) -> Rect {
        (*self).get_cutout(draw_data)
    }

    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        (*self).handle_input(response, handle);
    }
}

impl<T, D> Drawable for Vec<T>
where
    T: Drawable<DrawData = D>,
{
    type DrawData = D;

    fn draw(&mut self, handle: &mut CanvasHandle, draw_data: &Self::DrawData) {
        for drawable in self {
            drawable.draw(handle, draw_data);
        }
    }

    fn get_cutout(&mut self, draw_data: &Self::DrawData) -> Rect {
        if let Some(first) = self.first_mut() {
            let mut rect = first.get_cutout(draw_data);
            for drawable in self.iter_mut().skip(1) {
                rect = rect.union(drawable.get_cutout(draw_data));
            }
            rect
        } else {
            //dummy value
            Rect::from_two_pos((0.0, 0.0).into(), (10.0, 10.0).into())
        }
    }

    #[allow(unused_variables)]
    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        for drawable in self {
            drawable.handle_input(response, handle);
        }
    }
}

impl Drawable for () {
    type DrawData = ();

    fn draw(&mut self, _handle: &mut CanvasHandle, _draw_data: &Self::DrawData) {}

    fn get_cutout(&mut self, _draw_data: &Self::DrawData) -> Rect {
        //dummy value
        Rect::from_two_pos((0.0, 0.0).into(), (10.0, 10.0).into())
    }
}

impl<T, D> Drawable for Rc<RefCell<T>>
where
    T: Drawable<DrawData = D>,
{
    type DrawData = D;

    fn draw(&mut self, handle: &mut CanvasHandle, draw_data: &Self::DrawData) {
        let mut borrow = self.borrow_mut();
        borrow.draw(handle, draw_data);
    }

    fn get_cutout(&mut self, draw_data: &Self::DrawData) -> Rect {
        let mut borrow = self.borrow_mut();
        borrow.get_cutout(draw_data)
    }

    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        let mut borrow = self.borrow_mut();
        borrow.handle_input(response, handle);
    }
}

impl<T, D> Drawable for Box<T>
where
    T: Drawable<DrawData = D>,
{
    type DrawData = D;

    fn draw(&mut self, handle: &mut CanvasHandle, draw_data: &Self::DrawData) {
        self.deref_mut().draw(handle, draw_data);
    }

    fn get_cutout(&mut self, draw_data: &Self::DrawData) -> Rect {
        self.deref_mut().get_cutout(draw_data)
    }

    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        self.deref_mut().handle_input(response, handle);
    }
}

impl<T, G, D> Drawable for (T, G)
where
    T: Drawable<DrawData = D>,
    G: Drawable<DrawData = D>,
{
    type DrawData = D;

    fn draw(&mut self, handle: &mut CanvasHandle, draw_data: &Self::DrawData) {
        self.0.draw(handle, draw_data);
        self.1.draw(handle, draw_data);
    }

    fn get_cutout(&mut self, draw_data: &Self::DrawData) -> Rect {
        let rect0 = self.0.get_cutout(draw_data);
        let rect1 = self.1.get_cutout(draw_data);

        rect0.union(rect1)
    }

    #[allow(unused_variables)]
    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        self.0.handle_input(response, handle);
        self.1.handle_input(response, handle);
    }
}

pub struct Response {
    pub curser_pos: Option<Position>,
    pub clicked: bool,
}

impl From<&EGuiResponse> for Response {
    fn from(response: &EGuiResponse) -> Self {
        Response {
            curser_pos: response.hover_pos().map(Position::Gui),
            clicked: response.clicked(),
        }
    }
}
