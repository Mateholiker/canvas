use std::cell::RefCell;
use std::ops::DerefMut;
use std::rc::Rc;

use eframe::egui::{Rect, Response as EGuiResponse};

use crate::{CanvasHandle, Position};

pub trait Drawable {
    fn draw(&mut self, handle: &mut CanvasHandle);

    fn get_cutout(&mut self) -> Rect;

    #[allow(unused_variables)]
    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {}
}

impl<T> Drawable for Vec<T>
where
    T: Drawable,
{
    fn draw(&mut self, handle: &mut CanvasHandle) {
        for drawable in self {
            drawable.draw(handle);
        }
    }

    fn get_cutout(&mut self) -> Rect {
        if let Some(first) = self.first_mut() {
            let mut rect = first.get_cutout();
            for drawable in self {
                rect = rect.union(drawable.get_cutout());
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
    fn draw(&mut self, _handle: &mut CanvasHandle) {}

    fn get_cutout(&mut self) -> Rect {
        //dummy value
        Rect::from_two_pos((0.0, 0.0).into(), (10.0, 10.0).into())
    }
}

impl<T> Drawable for Rc<RefCell<T>>
where
    T: Drawable,
{
    fn draw(&mut self, handle: &mut CanvasHandle) {
        let mut borrow = self.borrow_mut();
        borrow.draw(handle);
    }

    fn get_cutout(&mut self) -> Rect {
        let mut borrow = self.borrow_mut();
        borrow.get_cutout()
    }

    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        let mut borrow = self.borrow_mut();
        borrow.handle_input(response, handle);
    }
}

impl<T> Drawable for Box<T>
where
    T: Drawable,
{
    fn draw(&mut self, handle: &mut CanvasHandle) {
        self.deref_mut().draw(handle);
    }

    fn get_cutout(&mut self) -> Rect {
        self.deref_mut().get_cutout()
    }

    fn handle_input(&mut self, response: &Response, handle: &CanvasHandle) {
        self.deref_mut().handle_input(response, handle);
    }
}

impl<T, G> Drawable for (T, G)
where
    T: Drawable,
    G: Drawable,
{
    fn draw(&mut self, handle: &mut CanvasHandle) {
        self.0.draw(handle);
        self.1.draw(handle);
    }

    fn get_cutout(&mut self) -> Rect {
        let rect0 = self.0.get_cutout();
        let rect1 = self.1.get_cutout();

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
