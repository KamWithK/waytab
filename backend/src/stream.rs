use std::os::fd::{AsRawFd, OwnedFd};

use anyhow::Result;
use ashpd::desktop::{
    PersistMode,
    screencast::{CursorMode, Screencast, SourceType, Stream as ScreencastStream},
};
use gstreamer::{self as gst, glib::MainLoop};
use gstreamer_rtsp_server::{
    self as gst_rtsp_server,
    prelude::{RTSPMediaFactoryExt, RTSPMountPointsExt, RTSPServerExt, RTSPServerExtManual},
};

pub(crate) async fn open_portal() -> Result<(ScreencastStream, OwnedFd)> {
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
        .ok_or_else(|| anyhow::anyhow!("No stream found or selected"))?
        .to_owned();

    let fd = proxy.open_pipe_wire_remote(&session).await?;

    Ok((stream, fd))
}

pub(crate) async fn initiate_stream() -> Result<()> {
    let (stream, stream_fd) = open_portal().await?;
    let pipewire_node_id = &stream.pipe_wire_node_id();
    let stream_raw_fd = &stream_fd.as_raw_fd();

    println!("node id {}, fd {}", pipewire_node_id, stream_raw_fd);

    let pipeline = format!(
        "pipewiresrc fd={stream_raw_fd} path={pipewire_node_id} min-buffers=1 max-buffers=4 ! vapostproc ! vah265enc ! rtph265pay name=pay0"
    );
    println!("{}", &pipeline);

    tokio::task::spawn_blocking(move || -> Result<()> {
        let main_loop = MainLoop::new(None, false);

        gst::init()?;
        let server = gst_rtsp_server::RTSPServer::new();
        server.set_address("0.0.0.0");

        let mounts = server
            .mount_points()
            .ok_or_else(|| anyhow::anyhow!("No RTSP server mounts found"))?;
        let factory = gst_rtsp_server::RTSPMediaFactory::new();
        factory.set_shared(true);
        factory.set_launch(&pipeline);
        factory.set_buffer_size(0);
        factory.set_latency(0u32);

        let id = server.attach(None)?;
        mounts.add_factory("/", factory);

        main_loop.run();
        id.remove();

        Ok(())
    })
    .await??;

    Ok(())
}
