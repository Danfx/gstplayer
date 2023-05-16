use gst::{prelude::*, Element, Pipeline};
use gtk::{gdk,glib};

#[derive(glib::Downgrade)]
pub struct GstManager {
    gtksink: Element,
    pipeline: Pipeline,
}

impl GstManager {

    pub fn new() -> Self {
        Self {
            gtksink: gst::ElementFactory::make("gtk4paintablesink").property("sync", false).build().unwrap(),
            pipeline: gst::Pipeline::new(),
        }
    }

    pub fn get_pipeline(&self) -> &Pipeline {
        &self.pipeline
    }

    pub fn get_paintable_sink(&self) -> gdk::Paintable {
        self.gtksink.property::<gdk::Paintable>("paintable")
    }

    pub fn set_video_filename(&self,filename:Option<&str>){
        self.build_pipeline(filename);
    }

    pub fn set_play_stream(&self){
        self.pipeline
            .set_state(gst::State::Playing)
            .expect("Unable to set the pipeline to the `Playing` state");
    }

    pub fn set_stop_stream(&self){
        self.pipeline
            .set_state(gst::State::Null)
            .expect("Unable to set the pipeline to the `Null` state");
    }

    fn build_pipeline(&self,filename:Option<&str>){
        let filesrc = gst::ElementFactory::make("filesrc")
            .property("location", filename)
            .build()
            .unwrap();
        let decodebin = gst::ElementFactory::make("decodebin").build().unwrap();

        let binsink = gst::Bin::default();
        let tee = gst::ElementFactory::make("tee").build().unwrap();
        
        // videosink
        let queue0 = gst::ElementFactory::make("queue").build().unwrap();
        let videoconvert = gst::ElementFactory::make("videoconvert").build().unwrap();
        let videoscale = gst::ElementFactory::make("videoscale").build().unwrap();

        // udpsink
        let queue1 = gst::ElementFactory::make("queue").build().unwrap();
        let avenc_mpeg4 = gst::ElementFactory::make("avenc_mpeg4").build().unwrap();
        let mpegtsmux = gst::ElementFactory::make("mpegtsmux").build().unwrap();
        let rtpmp2tpay = gst::ElementFactory::make("rtpmp2tpay").build().unwrap();
        let udpsink = gst::ElementFactory::make("udpsink")
            .property("host", "127.0.0.1")
            .property("port", 5000)
            .property("sync", false)
            .build().unwrap();

        // tcpsink
        let queue2 = gst::ElementFactory::make("queue").build().unwrap();
        let theoraenc = gst::ElementFactory::make("theoraenc").build().unwrap();
        let oggmux = gst::ElementFactory::make("oggmux").build().unwrap();
        let tcpserversink = gst::ElementFactory::make("tcpserversink")
            .property("host", "127.0.0.1")
            .property("port", 8180)
            .property("sync", false)
            .build().unwrap();

        binsink.add_many([
            &tee,
            &queue0,&videoconvert,&videoscale,&self.gtksink,
            &queue1,&avenc_mpeg4,&mpegtsmux,&rtpmp2tpay,&udpsink,
            &queue2,&theoraenc,&oggmux,&tcpserversink
        ]).unwrap();

        gst::Element::link_many([&tee,&queue0,&videoconvert,&videoscale,&self.gtksink]).unwrap();
        gst::Element::link_many([&tee,&queue1,&avenc_mpeg4,&mpegtsmux,&rtpmp2tpay,&udpsink]).unwrap();
        gst::Element::link_many([&tee,&queue2,&theoraenc,&oggmux,&tcpserversink]).unwrap();

        binsink.add_pad(&gst::GhostPad::with_target(&tee.static_pad("sink").unwrap()).unwrap()).unwrap();
        let sink = binsink.upcast();

        self.pipeline.add_many([&filesrc, &decodebin, &sink]).unwrap();
        gst::Element::link_many([&filesrc, &decodebin]).unwrap();

        let pipeline_weak = self.pipeline.downgrade();
        let sink_weak = sink.downgrade();
        decodebin.connect_pad_added(move |_, src_pad| {
            println!("new pad {:?}", src_pad);
            let pipeline = match pipeline_weak.upgrade() {
                Some(pipeline) => pipeline,
                None => return,
            };
            let sink = match sink_weak.upgrade() {
                Some(sink) => sink,
                None => return,
            };
            pipeline.remove(&sink).unwrap();
            pipeline.add(&sink).unwrap();
            sink.sync_state_with_parent().unwrap();

            let sink_pad = sink.static_pad("sink").unwrap();
            src_pad.link(&sink_pad).unwrap();
        });
                
    }
    
}