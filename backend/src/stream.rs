use std::os::fd::{AsRawFd, OwnedFd};

use ashpd::desktop::{
    PersistMode,
    screencast::{CursorMode, Screencast, SourceType, Stream as ScreencastStream},
};
use gstreamer::{self as gst, glib};
use gstreamer_rtsp_server::{
    self as gst_rtsp_server,
    prelude::{RTSPMediaFactoryExt, RTSPMountPointsExt, RTSPServerExt, RTSPServerExtManual},
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

    let pipeline = format!(
        "pipewiresrc fd={stream_raw_fd} path={pipewire_node_id} min-buffers=1 max-buffers=4 ! vapostproc ! vah265enc ! rtph265pay name=pay0"
    );
    println!("{}", &pipeline);

    gst::init().expect("Unable to start gstreamer");
    let main_loop = glib::MainLoop::new(None, false);
    let server = gst_rtsp_server::RTSPServer::new();
    server.set_address("192.168.1.10");

    let mounts = server.mount_points().unwrap();
    let factory = gst_rtsp_server::RTSPMediaFactory::new();
    factory.set_shared(true);
    factory.set_launch(&pipeline);
    factory.set_buffer_size(0);
    factory.set_latency(0u32);

    mounts.add_factory("/", factory);

    let id = server.attach(None).unwrap();

    println!(
        "Stream ready at rtsp://{}:{}",
        server.address().unwrap().to_string(),
        server.bound_port()
    );

    main_loop.run();
    id.remove();
}
