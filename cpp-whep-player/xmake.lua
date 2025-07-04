includes("packages.lua")

add_requires("glib 2.84.3", {system = false})
-- @see https://gstreamer.freedesktop.org/src/gstreamer/
add_requires("gstreamer 1.26.3", {system = false})
add_requires("gst-plugins-bad 1.26.3", {system = false})
add_requires("gst-plugins-base 1.26.3", {system = false})
add_requires("libsoup 2.74.2", {system = false})
-- add_requires("glib-networking 2.80.0", {system = false})
-- add_requires("gsettings-desktop-schemas 47.0", {system = false})

target("player", function()
    set_kind("binary")
    add_files("main.cc")
    add_packages("glib", "gstreamer", "gst-plugins-base", "gst-plugins-bad")
end)
