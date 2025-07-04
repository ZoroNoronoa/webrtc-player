#define GST_USE_UNSTABLE_API 1  // Removes compile warning

#include <glib.h>
#include <gst/gst.h>
#include <gst/sdp/sdp.h>
#include <gst/webrtc/webrtc.h>
#include <libsoup/soup.h>
#include <stdio.h>
#include <stdlib.h>

#include <csignal>
#include <cstdint>
#include <iostream>
#include <string>
#include <vector>

#include "logger.h"  // NOLINT

struct CustomData {
  GstElement* webrtc_source;  // 用于 WebRTC 信令和媒体流处理
  GstElement* pipeline;       // GStreamer 流水线
  GstElement* rtp_depay_vp8;  // 用于处理 VP8 格式的 RTP 数据包
  GstElement* vp8_decoder;    // 指向 VP8 解码器元素
  GstElement* sinkElement;    // 指向视频输出 （如 glimagesink） 元素

  std::string sdpOffer;   // 保存 SDP offer 字符串
  std::string sdpAnswer;  // 保存 SDP answer 字符串
  std::string location;
  std::string whepURL;  // 保存 WHEP 资源的 URL

  CustomData()
      : webrtc_source(nullptr), pipeline(nullptr), rtp_depay_vp8(nullptr), vp8_decoder(nullptr), sinkElement(nullptr) {
  }

  ~CustomData() {
    LOG_INFO("Destructing resources...");
    if (pipeline) {
      g_object_unref(pipeline);
    }
  }
};

GMainLoop* mainLoop = nullptr;
void padAddedHandler(GstElement* src, GstPad* pad, CustomData* data);
void onAnswerCreatedCallback(GstPromise* promise, gpointer userData);
void onRemoteDescSetCallback(GstPromise* promise, gpointer userData);
void onNegotiationNeededCallback(GstElement* src, CustomData* data);

void intSignalHandler(int32_t) {
  g_main_loop_quit(mainLoop);
}

void handleSDPs(CustomData* data) {
  GstSDPMessage* offerMessage;
  GstWebRTCSessionDescription* offerDesc;

  if (gst_sdp_message_new_from_text(data->sdpOffer.c_str(), &offerMessage) != GST_SDP_OK) {
    LOG_ERROR("Unable to create SDP object from offer");
  }

  offerDesc = gst_webrtc_session_description_new(GST_WEBRTC_SDP_TYPE_OFFER, offerMessage);
  if (!offerDesc) {
    LOG_ERROR("Unable to create SDP object from offer msg");
  }

  GstPromise* promiseRemote = gst_promise_new_with_change_func(onRemoteDescSetCallback, data, nullptr);
  if (!data->webrtc_source) {
    LOG_ERROR("webrtc_source is NULL");
  }

  g_signal_emit_by_name(data->webrtc_source, "set-remote-description", offerDesc, promiseRemote);
}

void getPostOffer(CustomData* const data) {
  SoupSession* session = ::soup_session_new();

  SoupMessage* msg = ::soup_message_new("POST", data->whepURL.c_str());
  ::soup_message_set_request(msg, "application/sdp", SOUP_MEMORY_STATIC, "", 0);

  if (!msg) {
    LOG_ERROR("NULL msg in getPostOffer()");
    exit(EXIT_FAILURE);
  }
  auto statusCode = soup_session_send_message(session, msg);

  if (statusCode != 200 && statusCode != 201) {
    LOG_ERROR("(%d):%s\n", statusCode, msg->response_body->data);
    exit(EXIT_FAILURE);
  }

  const char* location = soup_message_headers_get_one(msg->response_headers, "location");
  std::string sdpOffer(msg->response_body->data);

  data->location = location;
  data->sdpOffer = sdpOffer;

  g_object_unref(msg);
  g_object_unref(session);
}

void patchAnswer(CustomData* data) {
  SoupSession* session = soup_session_new();
  SoupMessage* msg = soup_message_new("PATCH", data->location.c_str());
  if (!msg) {
    LOG_ERROR("when creating msg in patchAnswer()");
    exit(EXIT_FAILURE);
  }
  const char* sdp = data->sdpAnswer.c_str();

  soup_message_set_request(msg, "application/sdp", SOUP_MEMORY_COPY, sdp, strlen(sdp));
  auto statusCode = soup_session_send_message(session, msg);

  // Cleanup
  g_object_unref(msg);
  g_object_unref(session);

  if (statusCode != 204) {
    LOG_ERROR("(%d):%s", statusCode, msg->response_body->data);
    exit(EXIT_FAILURE);
  }
}

int32_t main(int32_t argc, char** argv) {
  CustomData data;

  if (argc < 2) {
    std::cout << "Usage: GST_PLUGIN_PATH=/usr/lib/x86_64-linux-gnu/gstreamer1.0/gstreamer-1.0 ./whep-play WHEP-URL"
              << std::endl;
    return 1;
  }

  data.whepURL = argv[1];
  getPostOffer(&data);

  LOG_INFO("WHEP SDP Offer:\n%s", data.sdpOffer.c_str());
  LOG_INFO("WHEP Location: %s", data.location.c_str());

  gst_init(nullptr, nullptr);

  // Make elements
  data.webrtc_source = gst_element_factory_make("webrtcbin", "source");
  if (!data.webrtc_source) {
    LOG_ERROR("Failed to make element source. Note: GST_PLUGIN_PATH needs to be set");
    return 1;
  }

  data.sinkElement = gst_element_factory_make("glimagesink", "gli_sink");
  if (!data.sinkElement) {
    LOG_ERROR("Failed to make element gli_sink");
    return 1;
  }

  data.rtp_depay_vp8 = gst_element_factory_make("rtpvp8depay", "rtp_depayloader_vp8");
  if (!data.rtp_depay_vp8) {
    LOG_ERROR("Failed to make element rtp_depayloader_vp8");
    return 1;
  }

  data.vp8_decoder = gst_element_factory_make("vp8dec", "vp8_decoder");
  if (!data.vp8_decoder) {
    LOG_ERROR("Failed to make element vp8_decoder");
    return 1;
  }

  data.pipeline = gst_pipeline_new("pipeline");
  if (!data.pipeline) {
    LOG_ERROR("Failed to make element pipeline");
    return 1;
  }

  // Add elements
  if (!gst_bin_add(GST_BIN(data.pipeline), data.webrtc_source)) {
    LOG_ERROR("Failed to add element source. Note: GST_PLUGIN_PATH needs to be set");
    return 1;
  }

  if (!gst_bin_add(GST_BIN(data.pipeline), data.rtp_depay_vp8)) {
    LOG_ERROR("Failed to add element rtp_depayloader_vp8");
    return 1;
  }

  if (!gst_bin_add(GST_BIN(data.pipeline), data.vp8_decoder)) {
    LOG_ERROR("Failed to add element vp8_decoder");
    return 1;
  }

  if (!gst_bin_add(GST_BIN(data.pipeline), data.sinkElement)) {
    LOG_ERROR("Failed to add element gli_sink");
    return 1;
  }

  // Signals
  g_signal_connect(data.webrtc_source, "pad-added", G_CALLBACK(padAddedHandler), &data);
  g_signal_connect(data.webrtc_source, "on-negotiation-needed", G_CALLBACK(onNegotiationNeededCallback), &data);

  {
    struct sigaction sigactionData = {};
    sigactionData.sa_handler = intSignalHandler;
    sigactionData.sa_flags = 0;
    sigemptyset(&sigactionData.sa_mask);
    sigaction(SIGINT, &sigactionData, nullptr);
  }

  // Start playing
  LOG_INFO("Start playing...");
  if (gst_element_set_state(data.pipeline, GST_STATE_PLAYING) == GST_STATE_CHANGE_FAILURE) {
    LOG_ERROR("Unable to set the pipeline to the playing state");
    return 1;
  }

  LOG_INFO("Looping...");
  mainLoop = g_main_loop_new(nullptr, FALSE);
  g_main_loop_run(mainLoop);

  // Free resources - See CustomData destructor
  g_main_loop_unref(mainLoop);
  gst_element_set_state(data.pipeline, GST_STATE_NULL);
  gst_deinit();
  return 0;
}

void padAddedHandler(GstElement* src, GstPad* new_pad, CustomData* data) {
  LOG_INFO("Received new pad [%s] from [%s]", GST_PAD_NAME(new_pad), GST_ELEMENT_NAME(src));

  if (!gst_element_link_many(src, data->rtp_depay_vp8, data->vp8_decoder, data->sinkElement, nullptr)) {
    LOG_ERROR("Failed to link source to sink");
  }
}

void onNegotiationNeededCallback(GstElement* src, CustomData* data) {
  handleSDPs(data);
}

void onRemoteDescSetCallback(GstPromise* promise, gpointer userData) {
  auto data = reinterpret_cast<CustomData*>(userData);

  if (gst_promise_wait(promise) != GST_PROMISE_RESULT_REPLIED) {
    LOG_ERROR("onRemoteDescSetCallback: Failed to receive promise reply");
    exit(EXIT_FAILURE);
  }
  gst_promise_unref(promise);

  GstPromise* promiseAnswer = gst_promise_new_with_change_func(onAnswerCreatedCallback, data, nullptr);
  g_signal_emit_by_name(data->webrtc_source, "create-answer", nullptr, promiseAnswer);
}

void onAnswerCreatedCallback(GstPromise* promise, gpointer userData) {
  auto data = reinterpret_cast<CustomData*>(userData);

  GstWebRTCSessionDescription* answerPointer = nullptr;

  if (gst_promise_wait(promise) != GST_PROMISE_RESULT_REPLIED) {
    LOG_ERROR("onAnswerCreatedCallback: Failed to receive promise reply");
    exit(EXIT_FAILURE);
  }
  const GstStructure* reply = gst_promise_get_reply(promise);

  gst_promise_unref(promise);

  gst_structure_get(reply, "answer", GST_TYPE_WEBRTC_SESSION_DESCRIPTION, &answerPointer, nullptr);
  if (!answerPointer->sdp) {
    LOG_ERROR("No answer sdp!");
  }
  LOG_INFO("Get answer sdp:\n");
  std::cout << answerPointer->sdp << std::endl;

  g_signal_emit_by_name(data->webrtc_source, "set-local-description", answerPointer, nullptr);
  data->sdpAnswer = gst_sdp_message_as_text(answerPointer->sdp);
  patchAnswer(data);
}
