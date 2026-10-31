gtk4_dep = dependency('gtk4', required: get_option('gtk4'))

if gtk4_dep.found()
  shared_library('kime-gtk4',
    'src/gtk4.c',
    '../gtk3/src/immodule.c',
    '../gtk3/src/str_buf.c',
    dependencies: [gtk4_dep, kime_engine_dep],
    install: true,
    install_dir: get_option('libdir') / 'gtk-4.0' / '4.0.0' / 'immodules',
  )
endif
