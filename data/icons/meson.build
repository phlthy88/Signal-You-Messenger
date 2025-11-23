# Icons meson build file

icon_sizes = ['16', '24', '32', '48', '64', '128', '256', '512']

foreach size : icon_sizes
  install_data(
    'hicolor' / size + 'x' + size / 'apps' / 'com.signalyou.Messenger.png',
    install_dir: get_option('datadir') / 'icons' / 'hicolor' / size + 'x' + size / 'apps',
    rename: application_id + '.png',
  )
endforeach

# Scalable icon
install_data(
  'hicolor' / 'scalable' / 'apps' / 'com.signalyou.Messenger.svg',
  install_dir: get_option('datadir') / 'icons' / 'hicolor' / 'scalable' / 'apps',
  rename: application_id + '.svg',
)

# Symbolic icon
install_data(
  'hicolor' / 'symbolic' / 'apps' / 'com.signalyou.Messenger-symbolic.svg',
  install_dir: get_option('datadir') / 'icons' / 'hicolor' / 'symbolic' / 'apps',
  rename: application_id + '-symbolic.svg',
)
