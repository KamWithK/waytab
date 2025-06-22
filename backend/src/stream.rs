use std::os::fd::{AsRawFd, OwnedFd};

use ashpd::desktop::{
    PersistMode,
    screencast::{CursorMode, Screencast, SourceType, Stream as ScreencastStream},
};
use gstreamer::{
    self as gst,
    prelude::{ElementExt, GstBinExtManual, GstObjectExt},
};

pub(crate) async fn open_portal() -> ashpd::Result<(ScreencastStream, OwnedFd)> {
    let proxy = Screencast::new().await?;
    let session = proxy.create_session().await?;
    proxy
        .select_sources(
            &session,
            CursorMode::Embedded,
            SourceType::Monitor | SourceType::Window | SourceType::Virtual,
            false,
            None,
            PersistMode::ExplicitlyRevoked,
        )
        .await?;

    let response = proxy.start(&session, None).await?.response()?;
    let stream = response
        .streams()
        .first()
        .expect("No stream found or selected")
        .to_owned();

    let fd = proxy.open_pipe_wire_remote(&session).await?;

    Ok((stream, fd))
}

pub(crate) async fn initiate_stream() {
    let (stream, stream_fd) = open_portal().await.expect("failed to open portal");
    let pipewire_node_id = &stream.pipe_wire_node_id();
    let stream_raw_fd = &stream_fd.as_raw_fd();

    println!("node id {}, fd {}", pipewire_node_id, stream_raw_fd);

    gst::init().expect("Unable to start gstreamer");

    let pipewire_element = gst::ElementFactory::make("pipewiresrc")
        .property("fd", stream_raw_fd)
        .property("path", pipewire_node_id.to_string())
        .property("do-timestamp", true)
        .property("keepalive-time", 100)
        .build()
        .unwrap();

    let wayland_sink = gst::ElementFactory::make("waylandsink").build().unwrap();

    let convert = gst::ElementFactory::make("videoconvert").build().unwrap();

    let pipeline = gst::Pipeline::default();
    pipeline
        .add_many([&pipewire_element, &convert, &wayland_sink])
        .expect("Failed to add elements to pipeline");
    gst::Element::link_many([&pipewire_element, &convert, &wayland_sink])
        .expect("Failed to link elements");

    pipeline
        .set_state(gst::State::Playing)
        .expect("Failed to start pipeline");

    let bus = pipeline.bus().unwrap();

    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => {
                println!("EOS");
                break;
            }
            MessageView::Error(err) => {
                pipeline.set_state(gst::State::Null).unwrap();
                eprintln!(
                    "Got error from {}: {} ({})",
                    msg.src()
                        .map(|s| String::from(s.path_string()))
                        .unwrap_or_else(|| "None".into()),
                    err.error(),
                    err.debug().unwrap_or_else(|| "".into()),
                );
                break;
            }
            _ => (),
        }
    }
}
