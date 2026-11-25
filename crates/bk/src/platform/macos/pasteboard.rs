use std::path::PathBuf;

use objc2::AnyThread;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::{
    NSBitmapImageFileType, NSBitmapImageRep, NSDraggingItem, NSImage, NSPasteboard,
    NSPasteboardItem, NSPasteboardTypeFileURL, NSPasteboardTypeHTML, NSPasteboardTypePNG,
    NSPasteboardTypeString, NSPasteboardTypeTIFF, NSPasteboardWriting, NSWorkspace,
};
use objc2_foundation::{NSArray, NSData, NSDictionary, NSPoint, NSRect, NSSize, NSString};

use crate::drag::DragPayload;
use crate::portal::{file_path, file_uri};

fn board(name: Option<&str>) -> Retained<NSPasteboard> {
    match name {
        Some(name) => NSPasteboard::pasteboardWithName(&NSString::from_str(name)),
        None => NSPasteboard::generalPasteboard(),
    }
}

fn item_with(
    entries: &[(&NSString, &str)],
    data: &[(&NSString, &[u8])],
) -> Retained<NSPasteboardItem> {
    let item = NSPasteboardItem::new();
    for (kind, text) in entries {
        item.setString_forType(&NSString::from_str(text), kind);
    }
    for (kind, bytes) in data {
        item.setData_forType(&NSData::with_bytes(bytes), kind);
    }
    item
}

fn tiff_of(png: &[u8]) -> Option<Retained<NSData>> {
    NSImage::initWithData(NSImage::alloc(), &NSData::with_bytes(png))?.TIFFRepresentation()
}

/// The items of a payload as AppKit holds them on the pasteboard and in a drag session.
pub(crate) fn items(payload: &DragPayload) -> Vec<Retained<NSPasteboardItem>> {
    // SAFETY: these are AppKit's own constant strings, valid for the life of the process.
    let (string, html, file, png, tiff) = unsafe {
        (
            NSPasteboardTypeString,
            NSPasteboardTypeHTML,
            NSPasteboardTypeFileURL,
            NSPasteboardTypePNG,
            NSPasteboardTypeTIFF,
        )
    };
    match payload {
        DragPayload::Text(text) => vec![item_with(&[(string, text)], &[])],
        DragPayload::Html { text, html: markup } => {
            vec![item_with(&[(string, text), (html, markup)], &[])]
        }
        DragPayload::Image(bytes) => {
            let item = item_with(&[], &[(png, bytes)]);
            if let Some(tiff_bytes) = tiff_of(bytes) {
                item.setData_forType(&tiff_bytes, tiff);
            }
            vec![item]
        }
        DragPayload::Files(paths) => paths
            .iter()
            .map(|path| {
                item_with(
                    &[(file, &file_uri(path)), (string, &path.to_string_lossy())],
                    &[],
                )
            })
            .collect(),
    }
}

/// Replaces the pasteboard with the payload; `name` is a private pasteboard, `None` the general one.
pub(crate) fn write(name: Option<&str>, payload: &DragPayload) -> bool {
    let writers: Vec<_> = items(payload)
        .into_iter()
        .map(ProtocolObject::<dyn NSPasteboardWriting>::from_retained)
        .collect();
    let board = board(name);
    board.clearContents();
    board.writeObjects(&NSArray::from_retained_slice(&writers))
}

const DRAG_IMAGE: f64 = 32.0;

fn drag_image(payload: &DragPayload, index: usize) -> Option<Retained<NSImage>> {
    match payload {
        DragPayload::Image(bytes) => {
            NSImage::initWithData(NSImage::alloc(), &NSData::with_bytes(bytes))
        }
        DragPayload::Files(paths) => paths.get(index).map(|path| {
            NSWorkspace::sharedWorkspace().iconForFile(&NSString::from_str(&path.to_string_lossy()))
        }),
        DragPayload::Text(_) | DragPayload::Html { .. } => {
            NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &NSString::from_str("doc.text"),
                None,
            )
        }
    }
}

/// The items of a drag session, each drawn in a square under the pointer at `at` (view coordinates).
pub(crate) fn dragging_items(
    payload: &DragPayload,
    at: NSPoint,
) -> Retained<NSArray<NSDraggingItem>> {
    let frame = NSRect::new(
        NSPoint::new(at.x - DRAG_IMAGE / 2.0, at.y - DRAG_IMAGE / 2.0),
        NSSize::new(DRAG_IMAGE, DRAG_IMAGE),
    );
    let dragged: Vec<_> = items(payload)
        .into_iter()
        .enumerate()
        .map(|(index, item)| {
            let writer = ProtocolObject::<dyn NSPasteboardWriting>::from_retained(item);
            let dragging =
                NSDraggingItem::initWithPasteboardWriter(NSDraggingItem::alloc(), &writer);
            let image = drag_image(payload, index);
            // SAFETY: the contents are an NSImage, which AppKit draws, or nil.
            unsafe {
                dragging
                    .setDraggingFrame_contents(frame, image.as_deref().map(|image| image.as_ref()))
            };
            dragging
        })
        .collect();
    NSArray::from_retained_slice(&dragged)
}

fn png_of_tiff(tiff: &NSData) -> Option<Vec<u8>> {
    let representation = NSBitmapImageRep::imageRepWithData(tiff)?;
    // SAFETY: an empty dictionary is a valid set of properties for PNG.
    let png = unsafe {
        representation
            .representationUsingType_properties(NSBitmapImageFileType::PNG, &NSDictionary::new())
    }?;
    Some(png.to_vec())
}

pub(crate) fn files(board: &NSPasteboard) -> Vec<PathBuf> {
    // SAFETY: an AppKit constant string, valid for the life of the process.
    let file = unsafe { NSPasteboardTypeFileURL };
    board
        .pasteboardItems()
        .iter()
        .flatten()
        .filter_map(|item| file_path(&item.stringForType(file)?.to_string()))
        .collect()
}

/// The richest payload on the pasteboard: files, an image (PNG), HTML with its text, or text.
pub(crate) fn read(name: Option<&str>) -> Option<DragPayload> {
    // SAFETY: these are AppKit's own constant strings, valid for the life of the process.
    let (string, html, png, tiff) = unsafe {
        (
            NSPasteboardTypeString,
            NSPasteboardTypeHTML,
            NSPasteboardTypePNG,
            NSPasteboardTypeTIFF,
        )
    };
    let board = board(name);
    let items = board.pasteboardItems()?;
    let files = files(&board);
    if !files.is_empty() {
        return Some(DragPayload::Files(files));
    }
    let first = items.firstObject()?;
    if let Some(bytes) = first.dataForType(png) {
        return Some(DragPayload::Image(bytes.to_vec()));
    }
    if let Some(bytes) = first.dataForType(tiff).and_then(|data| png_of_tiff(&data)) {
        return Some(DragPayload::Image(bytes));
    }
    let text = first.stringForType(string).map(|text| text.to_string());
    match (text, first.stringForType(html)) {
        (text, Some(markup)) => Some(DragPayload::Html {
            text: text.unwrap_or_default(),
            html: markup.to_string(),
        }),
        (Some(text), None) => Some(DragPayload::Text(text)),
        (None, None) => None,
    }
}
