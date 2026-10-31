gtk3_dep = dependency('gtk+-3.0', required: get_option('gtk3'))

if gtk3_dep.found()
  shared_library('im-kime',
    'src/gtk.c',
    'src/immodule.c',
    'src/str_buf.c',
    dependencies: [gtk3_dep, kime_engine_dep],
    install: true,
    install_dir: get_option('libdir') / 'gtk-3.0' / '3.0.0' / 'immodules',
  )
endif
