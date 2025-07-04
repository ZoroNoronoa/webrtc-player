package("gstreamer", function()
    set_homepage("https://gstreamer.freedesktop.org")
    set_description(
        "GStreamer is a development framework for creating applications like media players, video editors, streaming media broadcasters and so on")
    set_license("LGPL-2.0-or-later")

    add_urls(
        "https://gstreamer.freedesktop.org/src/gstreamer/gstreamer-$(version).tar.xz",
        {alias = "home"})
    add_versions("home:1.26.3",
                 "dc661603221293dccc740862425eb54fbbed60fb29d08c801d440a6a3ff82680")

    add_configs("tools", {
        description = "Build tools.",
        default = false,
        type = "boolean"
    })
    add_configs("libunwind", {
        description = "Use libunwind to generate backtraces",
        default = false,
        type = "boolean"
    })

    if is_plat("linux") then
        add_extsources("pacman::gstreamer", "apt::libgstreamer1.0-dev")
    elseif is_plat("macosx") then
        add_extsources("brew::gstreamer")
    elseif is_plat("mingw") and is_subhost("msys") then
        add_extsources("pacman::gstreamer")
    end

    add_deps("meson", "ninja")
    if is_plat("windows") then
        add_deps("pkgconf", "winflexbison")
    else
        add_deps("flex", "bison")
    end
    add_deps("glib")

    add_includedirs("include/gstreamer-1.0")

    on_load(function(package)
        if package:config("libunwind") then
            package:add("deps", "libunwind")
        end
        if not package:config("shared") then
            package:add("defines", "GST_STATIC_COMPILATION")
        end
    end)

    on_install("windows", "macosx", "linux", "cross", function(package)
        local configs = {
            "-Dexamples=disabled", "-Dbenchmarks=disabled", "-Dtests=disabled"
        }
        table.insert(configs, "-Dgst_debug=" ..
                         (package:is_debug() and "true" or "false"))
        table.insert(configs, "-Ddefault_library=" ..
                         (package:config("shared") and "shared" or "static"))
        table.insert(configs, "-Dlibunwind=" ..
                         (package:config("libunwind") and "enabled" or
                             "disabled"))
        table.insert(configs, "-Dtools=" ..
                         (package:config("tools") and "enabled" or "disabled"))

        local packagedeps = {}
        if not package:dep("glib"):config("shared") then
            table.insert(packagedeps, "libiconv")
        end
        if package:is_plat("windows", "macosx") then
            table.insert(packagedeps, "libintl")
        end
        import("package.tools.meson").install(package, configs,
                                              {packagedeps = packagedeps})
    end)

    on_test(function(package)
        assert(package:has_cfuncs("gst_init", {includes = "gst/gst.h"}))
    end)
end)

package("gst-plugins-base", function()
    set_homepage("https://gstreamer.freedesktop.org")

    add_urls(
        "https://gstreamer.freedesktop.org/src/gst-plugins-base/gst-plugins-base-$(version).tar.xz")
    add_versions("1.26.3",
                 "4ef9f9ef09025308ce220e2dd22a89e4c992d8ca71b968e3c70af0634ec27933")

    add_deps("meson", "ninja")
    add_deps("gstreamer")
    add_includedirs("include", "include/gstreamer-1.0")

    on_install("windows", function(package)
        local configs = {"-Dexamples=disabled", "-Dtests=disabled"}
        table.insert(configs, "-Ddefault_library=" ..
                         (package:config("shared") and "shared" or "static"))

        local packagedeps = {}
        if package:is_plat("windows", "macosx") then
            table.insert(packagedeps, "libintl")
        end
        import("package.tools.meson").install(package, configs,
                                              {packagedeps = packagedeps})
    end)
end)

package("gst-plugins-bad", function()
    set_homepage("https://gstreamer.freedesktop.org")

    add_urls(
        "https://gstreamer.freedesktop.org/src/gst-plugins-bad/gst-plugins-bad-$(version).tar.xz")
    add_deps("meson", "ninja")
    add_deps("gstreamer")
    add_deps("gst-plugins-base")
    add_versions("1.26.3",
                 "95c48dacaf14276f4e595f4cbca94b3cfebfc22285e765e2aa56d0a7275d7561")
    add_includedirs("include", "include/gstreamer-1.0")

    on_install("windows", function(package)
        local configs = {"-Dexamples=disabled", "-Dtests=disabled"}
        table.insert(configs, "-Ddefault_library=" ..
                         (package:config("shared") and "shared" or "static"))

        local packagedeps = {}
        if package:is_plat("windows", "macosx") then
            table.insert(packagedeps, "libintl")
            table.insert(packagedeps, "gst-plugins-base") -- 不填的话会丢失头文件搜索路径
        end
        import("package.tools.meson").install(package, configs,
                                              {packagedeps = packagedeps})
    end)
end)

package("gsettings-desktop-schemas", function()
    add_urls("https://gitlab.gnome.org/GNOME/gsettings-desktop-schemas.git")
    add_deps("meson", "ninja")
    add_deps("glib")
    on_install("windows", function(package)
        local configs = {"-Dintrospection=false"}
        table.insert(configs, "-Ddefault_library=" ..
                         (package:config("shared") and "shared" or "static"))

        local packagedeps = {}
        import("package.tools.meson").install(package, configs,
                                              {packagedeps = packagedeps})
    end)
end)

package("glib-networking", function()
    -- https://download.gnome.org/sources/glib-networking/2.80/glib-networking-2.80.0.tar.xz
    add_urls(
        "https://download.gnome.org/sources/glib-networking/2.80/glib-networking-$(version).tar.xz")
    add_versions("2.80.0",
                 "d8f4f1aab213179ae3351617b59dab5de6bcc9e785021eee178998ebd4bb3acf")
    add_deps("gsettings-desktop-schemas 47.0")

    add_deps("meson", "ninja")
    on_install("windows", function(package)
        local configs = {"-Dlibproxy=disabled"}
        table.insert(configs, "-Ddefault_library=" ..
                         (package:config("shared") and "shared" or "static"))

        local packagedeps = {}
        import("package.tools.meson").install(package, configs,
                                              {packagedeps = packagedeps})
    end)
end)

package("libsoup", function()
    set_homepage("https://wiki.gnome.org/Projects/libsoup")

    add_urls("https://gitlab.gnome.org/GNOME/libsoup.git")
    add_deps("meson", "ninja")
    add_deps("gstreamer")
    add_deps("nghttp2")
    add_deps("sqlite3")
    add_deps("libpsl")

    on_install("windows", function(package)
        local configs = {"-Dtests=false"}
        table.insert(configs, "-Ddefault_library=" ..
                         (package:config("shared") and "shared" or "static"))

        local packagedeps = {}
        import("package.tools.meson").install(package, configs,
                                              {packagedeps = packagedeps})
    end)
end)
