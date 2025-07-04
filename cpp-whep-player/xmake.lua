includes("packages.lua")

add_rules("mode.release")
add_rules("plugin.compile_commands.autoupdate", {outputdir = "."})

if is_plat("windows") then
    add_requires("glib 2.84.3", {system = false})
    -- @see https://gstreamer.freedesktop.org/src/gstreamer/
    add_requires("gstreamer 1.26.3", {system = false})
    add_requires("gst-plugins-bad 1.26.3", {system = false})
    add_requires("gst-plugins-base 1.26.3", {system = false})
    add_requires("libsoup 2.74.2", {system = false})
    -- add_requires("glib-networking 2.80.0", {system = false})
    -- add_requires("gsettings-desktop-schemas 47.0", {system = false})
elseif is_plat("linux") then
    add_linkdirs("/usr/lib/x86_64-linux-gnu")
    -- glib
    add_sysincludedirs("/usr/include/glib-2.0",
                       "/usr/lib/x86_64-linux-gnu/glib-2.0/include")
    add_syslinks("glib-2.0")
    -- gstreamer-1.0
    -- pkg-config --libs gstreamer-1.0
    add_sysincludedirs("/usr/include/gstreamer-1.0")
    add_syslinks("gstreamer-1.0", "gobject-2.0")
    -- libsoup-2.4
    -- pkg-config --libs libsoup-2.4
    add_sysincludedirs("/usr/include/libsoup-2.4")
    add_syslinks("soup-2.4", "gmodule-2.0", "pthread", "gio-2.0")
    -- gstreamer-webrtc-1.0
    -- pkg-config --libs gstreamer-webrtc-1.0
    add_syslinks("gstwebrtc-1.0", "gstbase-1.0")
    -- gstreamer-sdp-1.0
    -- pkg-config --libs gstreamer-sdp-1.0
    add_syslinks("gstsdp-1.0")
end

target("whep-player", function()
    set_kind("binary")
    add_files("src/main.cc")
end)
