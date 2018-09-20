#![feature(duration_as_u128)]
#![allow(unused_imports, unused_mut, unused_variables, dead_code)]

extern crate ffmpeg_sys;

pub use ffmpeg_sys::*;

use std::str;
use std::ptr;
use std::mem;
use std::slice;
use std::ffi::CStr;
use std::io::{ Write, };
use std::fs::{ self, OpenOptions, File, };


unsafe fn run() {
    let input_filename = "a.mp4".to_string();
    let output_filename = "b.mp4".to_string();

    let mut input_fmt_ctx: *mut AVFormatContext = avformat_alloc_context();
    let mut output_fmt_ctx: *mut AVFormatContext = avformat_alloc_context();

    if avformat_open_input(&mut input_fmt_ctx, 
                            input_filename.as_ptr() as *const i8,
                            0 as _,
                            0 as _) < 0 {
        panic!("Could not open input file {:?}", input_filename);
    }

    if avformat_find_stream_info(input_fmt_ctx, ptr::null_mut()) < 0 {
        panic!("Failed to retrieve input stream information");
    }

    av_dump_format(input_fmt_ctx, 0, input_filename.as_ptr() as *const i8, 0);
    avformat_alloc_output_context2(&mut output_fmt_ctx,
                            0 as _,
                            0 as _,
                            output_filename.as_ptr() as *const i8);
    if output_fmt_ctx.is_null() {
        panic!("Could not create output context");
    }
    
    let nb_stream = (*input_fmt_ctx).nb_streams;
    assert_eq!(nb_stream > 0, true);


    for i in 0..nb_stream {
        let input_stream: *mut AVStream = *(*input_fmt_ctx).streams.offset(i as isize);
        assert_eq!(input_stream.is_null(), false);

        let input_stream_codecpar: *mut AVCodecParameters = (*input_stream).codecpar;
        assert_eq!(input_stream_codecpar.is_null(), false);

        print!("Stream index: {:?}#{:?} codec: {:?}({:?}) width: {:?} height: {:?}",
                                (*input_stream).index,
                                (*input_stream).id,
                                (*input_stream_codecpar).codec_id,
                                (*input_stream_codecpar).codec_type,
                                (*input_stream_codecpar).width,
                                (*input_stream_codecpar).height );
        
        let input_stream_codec_type: AVMediaType = (*input_stream_codecpar).codec_type;

        if input_stream_codec_type == AVMediaType::AVMEDIA_TYPE_AUDIO {
            println!(" format: {:?}", mem::transmute::<i32, AVSampleFormat>((*input_stream_codecpar).format) );
        }
        if input_stream_codec_type == AVMediaType::AVMEDIA_TYPE_VIDEO {
            println!(" format: {:?}", mem::transmute::<i32, AVPixelFormat>((*input_stream_codecpar).format) );
            // println!("format: {:?}", (*input_stream_codecpar).format as AVPixelFormat);
        }

        if input_stream_codec_type != AVMediaType::AVMEDIA_TYPE_AUDIO &&
            input_stream_codec_type != AVMediaType::AVMEDIA_TYPE_VIDEO {
            continue;
        }

        let mut out_stream: *mut AVStream = avformat_new_stream(output_fmt_ctx, 0 as _);
        if out_stream.is_null() {
            panic!("Failed allocating output stream");
        }

        if avcodec_parameters_copy((*out_stream).codecpar, input_stream_codecpar as *const _) < 0 {
            panic!("Failed to copy codec parameters");
        }

        (*(*out_stream).codecpar).codec_tag = 0;
        (*(*out_stream).codecpar).codec_tag = (*(*input_stream).codecpar).codec_tag;
        (*out_stream).pts = (*input_stream).pts;
        (*out_stream).duration = (*input_stream).duration;
        (*out_stream).time_base = (*input_stream).time_base;
        (*out_stream).start_time = (*input_stream).start_time;    
    }

    av_dump_format(output_fmt_ctx, 0, output_filename.as_ptr() as *const i8, 1);

    let ofmt = (*output_fmt_ctx).oformat;
    if (*ofmt).flags & AVFMT_NOFILE <= 0 {
        if avio_open(&mut (*output_fmt_ctx).pb, output_filename.as_ptr() as *const i8, AVIO_FLAG_WRITE) < 0 {
            panic!("Could not open output file: {:?}", output_filename);
        }
    }

    let iformat: *mut AVInputFormat = (*input_fmt_ctx).iformat;
    let oformat: *mut AVOutputFormat = (*output_fmt_ctx).oformat;

    (*oformat).flags = AVFMT_FLAG_GENPTS;

    av_ret(avformat_write_header(output_fmt_ctx, ptr::null_mut()))
        .expect("Error occurred when opening output file");

    let mut got_picture_ptr = 0i32;
    let mut pkt: AVPacket = *av_packet_alloc();
    let mut output_frame: *mut AVFrame = av_frame_alloc();


    let input_stream: *mut AVStream = *(*input_fmt_ctx).streams.offset(0 as isize);
    let input_stream_codecpar: *mut AVCodecParameters = (*input_stream).codecpar;
    let input_codec_type: AVMediaType = (*input_stream_codecpar).codec_type;
    let input_codec_id: AVCodecID = (*input_stream_codecpar).codec_id;

    assert_eq!(input_codec_type, AVMediaType::AVMEDIA_TYPE_VIDEO);
    let input_codec: *mut AVCodec = avcodec_find_decoder(input_codec_id);
    assert_eq!(input_codec.is_null(), false);

    let input_codec_ctx: *mut AVCodecContext = avcodec_alloc_context3(input_codec);
    assert_eq!(input_codec_ctx.is_null(), false);
    
    av_ret(avcodec_parameters_to_context(input_codec_ctx, input_stream_codecpar))
        .expect("Copy codec parameter failed.");

    // Init the decoders, with or without reference counting
    let mut opts: *mut AVDictionary = ptr::null_mut();
    av_dict_set(&mut opts, "refcounted_frames".as_ptr() as *const i8, "1".as_ptr() as *const i8, 0);
    av_ret(avcodec_open2(input_codec_ctx, input_codec, &mut opts))
        .expect("Error open codec.");

    // YUV420P RawVideo File
    let _ = fs::remove_file("rawvideo.yuv420p");
    let mut rawvideo_file = OpenOptions::new().append(true).create(true).open("rawvideo.yuv420p").unwrap();

    loop {
        pkt.data = ptr::null_mut();
        pkt.size = 0;
        
        if av_read_frame(input_fmt_ctx, &mut pkt) < 0 {
            break;
        }

        let input_stream: *mut AVStream = *(*input_fmt_ctx).streams.offset(pkt.stream_index as isize);
        let output_stream: *mut AVStream = *(*output_fmt_ctx).streams.offset(pkt.stream_index as isize);
        
        // decode video frame
        if pkt.size > 0 && pkt.stream_index == 0 {
            av_ret(avcodec_decode_video2(input_codec_ctx, output_frame, &mut got_picture_ptr, &pkt))
                .expect("Error decoding video frame");

            // Save Picture
            let frame_index = (*output_frame).coded_picture_number;
            if got_picture_ptr > 0 && frame_index < 300 
                && (*output_frame).width == (*input_codec_ctx).width
                && (*output_frame).height == (*input_codec_ctx).height {

                println!("stream index: {:?} pkt size: {:?} frame width: {:?} frame height: {:?} got_picture_ptr: {:?}", 
                    pkt.stream_index,
                    pkt.size,
                    (*output_frame).width,
                    (*output_frame).height,
                    got_picture_ptr,);

                let y_buffer = slice::from_raw_parts_mut(
                    (*output_frame).data[0],
                    ((*output_frame).width * (*output_frame).height) as usize
                );
                let u_buffer = slice::from_raw_parts_mut(
                    (*output_frame).data[1],
                    ((*output_frame).width * (*output_frame).height * 3 / 2) as usize
                );
                let v_buffer = slice::from_raw_parts_mut(
                        (*output_frame).data[2],
                        ((*output_frame).width * (*output_frame).height * 3 / 2) as usize
                );
                for i in 0..(*output_frame).height {
                    let start = (i * (*output_frame).linesize[0]) as usize;
                    let end = start + (*output_frame).width as usize;
                    rawvideo_file.write(&y_buffer[start..end]).unwrap();
                }
                for i in 0..(*output_frame).height/2 {
                    let start = (i * (*output_frame).linesize[1]) as usize;
                    let end = start + ((*output_frame).width/2) as usize;
                    rawvideo_file.write(&u_buffer[start..end]).unwrap();
                }
                for i in 0..(*output_frame).height/2 {
                    let start = (i * (*output_frame).linesize[2]) as usize;
                    let end = start + ((*output_frame).width/2) as usize;
                    rawvideo_file.write(&v_buffer[start..end]).unwrap();
                }
                // Play:
                // ffplay -f rawvideo -pix_fmt yuv420p -video_size 1280x720 rawvideo.yuv420p
            }

            av_frame_unref(output_frame);

            // copy packet
            pkt.pts = av_rescale_q_rnd(pkt.pts, (*input_stream).time_base, (*output_stream).time_base,
                AVRounding::AV_ROUND_PASS_MINMAX );
            pkt.dts = av_rescale_q_rnd(pkt.dts, (*input_stream).time_base, (*output_stream).time_base, 
                AVRounding::AV_ROUND_PASS_MINMAX );
            pkt.duration = av_rescale_q(pkt.duration, (*input_stream).time_base, (*output_stream).time_base);
            pkt.pos = -1;

            av_ret(av_interleaved_write_frame(output_fmt_ctx, &mut pkt))
                .expect("Error muxing packet");

            av_packet_unref(&mut pkt);
        }
    }

    av_write_trailer(output_fmt_ctx);

    avformat_close_input(&mut input_fmt_ctx);
    // Close output
    if output_fmt_ctx.is_null() == false && !( (*ofmt).flags & AVFMT_NOFILE <= 0) {
        avio_closep(&mut (*output_fmt_ctx).pb);
    }
    avformat_free_context(output_fmt_ctx);
    av_frame_free(&mut output_frame);
}

unsafe fn decode_video_packet() {

    unimplemented!()
}

unsafe fn init() {
    av_register_all();
    avcodec_register_all();
    avfilter_register_all();
    avdevice_register_all();
}

fn av_ret(errnum: i32) -> Result<(), String> {
    pub const AV_ERROR_CHARS: [i8; AV_ERROR_MAX_STRING_SIZE] = [0i8; AV_ERROR_MAX_STRING_SIZE];

    if errnum < 0i32 {
        unsafe {
            let e = CStr::from_ptr(
                av_make_error_string(AV_ERROR_CHARS.as_mut_ptr(), AV_ERROR_MAX_STRING_SIZE, errnum))
                    .to_string_lossy()
                    .to_string();
            Err(e)
        }
    } else {
        Ok(())
    }
}

fn main() {
    // AV_LOG_INFO AV_LOG_WARNING AV_LOG_ERROR AV_LOG_DEBUG AV_LOG_FATAL AV_LOG_PANIC
    // AV_LOG_TRACE AV_LOG_VERBOSE
    // AV_LOG_QUIET 
    unsafe {
        av_log_set_level(AV_LOG_INFO);
        init();
        run();
    }
}