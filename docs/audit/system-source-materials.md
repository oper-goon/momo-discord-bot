# Runtime source materials review

Review date: 2026-11-24. Evidence is the completed amd64 Debian image's collected
`image-notices/inventory.json` and copied notices from the CI artifact. It records
**311 installed Debian packages** and **14 Python distributions**. This report
identifies source-material work; it is not license clearance, a source offer, or
proof that corresponding source has been supplied. No source was downloaded in
this review.

## What must be retained with a release

Use each recorded **Debian source package and exact source version**, including
epoch and Debian revision. For the applicable copyleft components, obtain the
matching Debian source package: its signed `.dsc`, referenced upstream tarballs,
Debian patch tarball/diff and build instructions. Verify their checksums and
retain the materials with the exact binary/image release. An upstream Git tag
alone can omit Debian patches and is not a substitute for this source package.

Record which binary packages each source package supplies. Include applicable
build/configuration scripts and review any LGPL relinking, reverse-engineering,
modification and installation-information requirements. Preserve actual notices
and exceptions. A bundled copyright file, installed shared library or Debian
mirror URL by itself does not establish that the distributor fulfilled source
requirements. Do not invent download URLs or substitute a newer source version.

The image's full inventory is the baseline, including ordinary shell/package
utilities inherited from Debian. Source obligations are not limited to tools
Momokun invokes. Do not apply PolyForm restrictions to third-party covered source.

## Immediate priorities from actual image evidence

| Source package | Exact source version | Installed components or evidence | Action |
| --- | --- | --- | --- |
| ffmpeg | `7:5.1.9-0+deb12u1` | ffmpeg and eight libav/libsw/libpostproc binary packages; shared build | Retain Debian source and configuration; map enabled codecs and linked libraries |
| x264 | `2:0.164.3095+gitbaee400-3` | libx264-164; FFmpeg enables libx264; GPL-2+ notice | Obtain matching source and preserve all component exceptions/notices |
| x265 | `3.5-2` | libx265-199; FFmpeg enables libx265; GPL-2+ notice | Obtain matching source |
| xvidcore | `2:1.3.7-1` | libxvidcore4; FFmpeg enables libxvid; GPL/LGPL notices | Obtain matching source |
| rubberband | `3.1.2+dfsg0-1` | librubberband2; FFmpeg enables librubberband; GPL-2+ notice | Obtain source and preserve mixed component notices |
| libcdio | `2.1.0-4` | libcdio19; copyright Files: * stanza GPL-3 | Confirm actual FFmpeg linkage/effective license and preserve GPL-3 source terms |
| libcdio-paranoia | `10.2+2.0.1-1` | libcdio-cdda2 and libcdio-paranoia2 | Review GPL-3 coverage and collect exact source |
| readline | `8.2-1.3` | libreadline8 and readline-common; GPL-3 notices | Collect exact source and inspect which shipped executables link it |
| bash | `5.2.15-2` | inherited shell; GPL-3+ main code | Include corresponding source and scripts, not only media components |
| coreutils | `9.1-1` | inherited command suite; GPL-3+ | Include corresponding source |
| findutils | `4.9.0-4` | inherited utilities; GPL-3+ | Include corresponding source |
| grep | `3.8-5` | inherited utility; GPL-3+ references | Inspect file-level stanzas and include applicable source |
| sed | `4.9-1+deb12u1` | inherited utility; GPL-3+ | Include corresponding source |
| tar | `1.34+dfsg-1.2+deb12u1` | inherited archiver; GPL-3+ | Include corresponding source |
| gzip | `1.12-1` | inherited compression tool; GPL-3 references | Inspect exact terms and include applicable source |
| diffutils | `1:3.8-4` | inherited tools; GPL-3+ | Include corresponding source |
| gnupg2 | `2.2.40-1.1+deb12u2` | gpgv; GPL-3+ references | Preserve applicable source and mixed notices |
| glibc | `2.36-9+deb12u14` | libc6 and libc-bin; library/system code | Review LGPL/source and relinking obligations with exact Debian package |
| gcc-12 | `12.2.0-14+deb12u1` | libgcc, libstdc++, libgomp, libgfortran, libquadmath and base package | Preserve runtime exceptions; do not treat all files as unrestricted or plain GPL |
| gnutls28 | `3.7.9-2+deb12u7` | libgnutls30; FFmpeg enables gnutls | Review actual LGPL/GPL alternatives and component notices |
| gmp | `2:6.2.1+dfsg1-1.1` | libgmp10; LGPL-3+/GPL alternatives | Record applicable licensing choice and retain source |

FFmpeg reports `--enable-gpl`, `--enable-shared` and the codec/library options
above. It also enables many other libraries listed in the complete candidate
manifest below. Its recorded configuration does **not** include
`--enable-version3` or `--enable-nonfree`. Do not infer the effective license
solely from those flags: GPL-3-covered libcdio is installed and enabled through
`--enable-libcdio`. Capture `ffmpeg -L`, actual shared-library dependency records
and the matching Debian build recipe to resolve the precise linkage/license
assessment. The inventory is not evidence that every installed library is linked
into FFmpeg or Momokun.

Copied Debian notices frequently reference `/usr/share/common-licenses/` instead
of containing full license text. Those referenced texts must remain available in
the image and accompany a standalone notice artifact where applicable. The
previous collector copied `/usr/share/doc` notices without that common-license
tree. The 2026-11-25 follow-up adds full common-license collection and captures
`ffmpeg -L`; the revised image still needs Linux CI validation.

## Python inventory and separate source requirements

| Distribution | Version | Observed packaged notice/metadata | Follow-up |
| --- | --- | --- | --- |
| bgutil-ytdlp-pot-provider | `2.0.2` | No matching packaged license text; license metadata null | Resolve from exact upstream artifact/commit with reviewed provenance |
| brotli | `1.2.1` | MIT text | Verify native wheel's Brotli component notices and exact build provenance |
| certifi | `2026.11.20` | MPL-2.0 notice | Supply corresponding covered source/data and recipient source information |
| charset-normalizer | `3.5.3` | MIT text | Check exact wheel and any compiled extension's component inventory |
| idna | `3.20` | BSD-3-Clause expression and license text | Retain notice; inspect bundled Unicode/data notices |
| mutagen | `1.48.1` | GPL-2.0-or-later expression and COPYING | Obtain exact source distribution and verify it is preferred source for installed files |
| pip | `23.0.1` | Top-level MIT text | Inspect vendored packages separately; one MIT text does not cover every bundled component |
| pycryptodomex | `3.24.2` | AUTHORS and BSD/public-domain license text | Preserve all original notices; inspect native wheel's vendored primitives and sources |
| requests | `2.34.2` | Apache-2.0 LICENSE and NOTICE | Preserve both files |
| setuptools | `66.1.1` | Top-level license text | Inspect its vendored dependencies and any additional component notices |
| urllib3 | `2.8.0` | MIT expression and text | Retain notice |
| websockets | `17.4` | BSD-3-Clause expression and text | Inspect native extension/build provenance if present |
| yt-dlp | `2026.10.26` | Unlicense expression and text | Verify vendored components/additional files in exact wheel; retain separate tool notices |
| yt-dlp-ejs | `0.8.0` | Expression: Unlicense AND MIT AND ISC; copied text is only Unlicense | Resolve missing MIT/ISC component notices from exact artifact before clearance |

The collector does not list wheel filenames, hashes, RECORD-native-file maps or
source distributions, so this review cannot establish exact native contents or
complete corresponding source. Retrieve the exact installed wheel/source
artifacts with verified hashes and inspect their file inventories before
publication. Keep Python package obligations separate from Debian packages with
similar names. In particular, the pip installation is represented by Debian
source `python-pip` version `23.0.1+dfsg-1`, but that alone does not map every
vendored Python module's requirements.

Deno is copied from another image, outside dpkg/Python inventory. Its own notices,
bundled runtime components and exact source provenance need separate review.
The optional provider container is another distribution and is not covered by
this runtime image's artifact.

## Conservative Debian source-package review manifest

The following table groups exact inventory records by source package/version
whose copied notices mention GPL/LGPL, either in license stanzas or prose:
**146 source groups**, with **48 containing a GPL-3 reference**. These are review
candidates, not 146 determinations that every installed binary is GPL-covered.
Copyright files often describe an entire source package, including tests, build
tools, documentation, alternative licenses and exceptions absent from a binary.
For example, a GPL build helper in an otherwise permissive library must not be
misreported as proof the library itself is GPL. Conversely, text searching can
miss unusual license descriptions. Reconcile file-level stanzas, installed files
and build records before finalizing source requirements. The full 311-package
inventory remains authoritative for installed version identity.

The GPL-3 column means **reference detected**, not effective license. No mutable
or guessed source download URLs are provided.

| Source package | Exact source version | Installed binary packages | GPL-3 reference |
| --- | --- | --- | --- |
| acl | `2.3.1-3` | libacl1:amd64 | No |
| adduser | `3.134` | adduser | No |
| alsa-lib | `1.2.8-1` | libasound2-data, libasound2:amd64 | No |
| apt | `2.6.2` | apt, libapt-pkg6.0:amd64 | No |
| attr | `1:2.5.1-4` | libattr1:amd64 | No |
| audit | `1:3.0.9-1` | libaudit-common, libaudit1:amd64 | No |
| base-files | `12.4+deb12u15` | base-files | No |
| base-passwd | `3.6.1` | base-passwd | No |
| bash | `5.2.15-2` | bash | Yes |
| bzip2 | `1.0.8-5` | libbz2-1.0:amd64 | No |
| ca-certificates | `20250419~deb12u2` | ca-certificates | No |
| cairo | `1.16.0-7` | libcairo-gobject2:amd64, libcairo2:amd64 | No |
| cdebconf | `0.270` | libdebconfclient0:amd64 | No |
| chromaprint | `1.5.1-2` | libchromaprint1:amd64 | No |
| codec2 | `1.0.5-1` | libcodec2-1.0:amd64 | No |
| coreutils | `9.1-1` | coreutils | Yes |
| dash | `0.5.12-2` | dash | No |
| db5.3 | `5.3.28+dfsg2-1` | libdb5.3:amd64 | Yes |
| dbus | `1.14.10-1~deb12u1` | libdbus-1-3:amd64 | No |
| debian-archive-keyring | `2023.3+deb12u2` | debian-archive-keyring | No |
| debianutils | `5.7-0.5~deb12u1` | debianutils | No |
| diffutils | `1:3.8-4` | diffutils | Yes |
| dpkg | `1.21.23` | dpkg | No |
| e2fsprogs | `1.47.0-2` | e2fsprogs, libcom-err2:amd64, libext2fs2:amd64, libss2:amd64, logsave | No |
| elfutils | `0.188-2.1` | libelf1:amd64 | Yes |
| ffmpeg | `7:5.1.9-0+deb12u1` | ffmpeg, libavcodec59:amd64, libavdevice59:amd64, libavfilter8:amd64, libavformat59:amd64, libavutil57:amd64, libpostproc56:amd64, libswresample4:amd64, libswscale6:amd64 | Yes |
| findutils | `4.9.0-4` | findutils | Yes |
| flac | `1.4.2+ds-2` | libflac12:amd64 | No |
| flite | `2.2-5` | libflite1:amd64 | Yes |
| fonts-dejavu | `2.37-6` | fonts-dejavu-core | No |
| freetype | `2.12.1+dfsg-5+deb12u4` | libfreetype6:amd64 | Yes |
| fribidi | `1.0.8-2.1` | libfribidi0:amd64 | No |
| game-music-emu | `0.6.3-6` | libgme0:amd64 | No |
| gcc-12 | `12.2.0-14+deb12u1` | gcc-12-base:amd64, libgcc-s1:amd64, libgfortran5:amd64, libgomp1:amd64, libquadmath0:amd64, libstdc++6:amd64 | Yes |
| gdk-pixbuf | `2.42.10+dfsg-1+deb12u4` | libgdk-pixbuf-2.0-0:amd64, libgdk-pixbuf2.0-common | No |
| glib2.0 | `2.74.6-2+deb12u9` | libglib2.0-0:amd64 | Yes |
| glibc | `2.36-9+deb12u14` | libc-bin, libc6:amd64 | No |
| gmp | `2:6.2.1+dfsg1-1.1` | libgmp10:amd64 | Yes |
| gnupg2 | `2.2.40-1.1+deb12u2` | gpgv | Yes |
| gnutls28 | `3.7.9-2+deb12u7` | libgnutls30:amd64 | Yes |
| graphite2 | `1.3.14-1+deb12u1` | libgraphite2-3:amd64 | No |
| grep | `3.8-5` | grep | Yes |
| gzip | `1.12-1` | gzip | Yes |
| harfbuzz | `6.0.0+dfsg-3` | libharfbuzz0b:amd64 | Yes |
| hostname | `3.23+nmu1` | hostname | No |
| icu | `72.1-3+deb12u1` | libicu72:amd64 | Yes |
| init-system-helpers | `1.65.2+deb12u1` | init-system-helpers | No |
| jackd2 | `1.9.21~dfsg-3` | libjack-jackd2-0:amd64 | Yes |
| jbigkit | `2.1-6.1` | libjbig0:amd64 | No |
| keyutils | `1.6.3-2` | libkeyutils1:amd64 | No |
| krb5 | `1.20.1-2+deb12u5` | libgssapi-krb5-2:amd64, libk5crypto3:amd64, libkrb5-3:amd64, libkrb5support0:amd64 | No |
| lame | `3.100-6` | libmp3lame0:amd64 | No |
| lcms2 | `2.14-2+deb12u1` | liblcms2-2:amd64 | Yes |
| libass | `1:0.17.1-1+deb12u1` | libass9:amd64 | No |
| libasyncns | `0.8-6` | libasyncns0:amd64 | No |
| libavc1394 | `0.5.4-5` | libavc1394-0:amd64 | No |
| libbluray | `1:1.3.4-1` | libbluray2:amd64 | No |
| libbs2b | `3.1.0+dfsg-7` | libbs2b0:amd64 | Yes |
| libcaca | `0.99.beta20-3+deb12u1` | libcaca0:amd64 | No |
| libcap-ng | `0.8.3-1` | libcap-ng0:amd64 | Yes |
| libcap2 | `1:2.66-4+deb12u3` | libcap2:amd64 | No |
| libcdio | `2.1.0-4` | libcdio19:amd64 | Yes |
| libcdio-paranoia | `10.2+2.0.1-1` | libcdio-cdda2:amd64, libcdio-paranoia2:amd64 | Yes |
| libdatrie | `0.2.13-2` | libdatrie1:amd64 | No |
| libdc1394 | `2.2.6-4` | libdc1394-25:amd64 | No |
| libdecor-0 | `0.1.1-2` | libdecor-0-0:amd64 | Yes |
| libffi | `3.4.4-1` | libffi8:amd64 | Yes |
| libgcrypt20 | `1.10.1-3+deb12u1` | libgcrypt20:amd64 | No |
| libglvnd | `1.6.0-1` | libgl1:amd64, libglvnd0:amd64, libglx0:amd64 | Yes |
| libgpg-error | `1.46-1` | libgpg-error0:amd64 | Yes |
| libidn2 | `2.3.3-1` | libidn2-0:amd64 | Yes |
| libiec61883 | `1.2.0-6` | libiec61883-0:amd64 | No |
| libnsl | `1.3.0-2` | libnsl2:amd64 | Yes |
| libopenmpt | `0.6.9-1` | libopenmpt0:amd64 | Yes |
| libpgm | `5.3.128~dfsg-2` | libpgm-5.3-0:amd64 | No |
| libplacebo | `4.208.0-3` | libplacebo208:amd64 | No |
| libpng1.6 | `1.6.39-2+deb12u6` | libpng16-16:amd64 | No |
| libraw1394 | `2.1.2-2` | libraw1394-11:amd64 | No |
| librsvg | `2.54.7+dfsg-1~deb12u1` | librsvg2-2:amd64 | No |
| libsamplerate | `0.2.2-3` | libsamplerate0:amd64 | Yes |
| libsdl2 | `2.26.5+dfsg-1` | libsdl2-2.0-0:amd64 | Yes |
| libseccomp | `2.5.4-1+deb12u1` | libseccomp2:amd64 | No |
| libselinux | `3.4-1` | libselinux1:amd64 | No |
| libsemanage | `3.4-1` | libsemanage-common, libsemanage2:amd64 | No |
| libsepol | `3.4-2.1` | libsepol2:amd64 | No |
| libsndfile | `1.2.0-1+deb12u1` | libsndfile1:amd64 | Yes |
| libsodium | `1.0.18-1+deb12u1` | libsodium23:amd64 | No |
| libsoxr | `0.1.3-4` | libsoxr0:amd64 | No |
| libssh | `0.10.6-0+deb12u3` | libssh-gcrypt-4:amd64 | No |
| libtasn1-6 | `4.19.0-2+deb12u1` | libtasn1-6:amd64 | Yes |
| libthai | `0.1.29-1` | libthai-data, libthai0:amd64 | No |
| libtirpc | `1.3.3+ds-1` | libtirpc-common, libtirpc3:amd64 | No |
| libudfread | `1.1.2-1` | libudfread0:amd64 | No |
| libunistring | `1.0-2` | libunistring2:amd64 | Yes |
| libusb-1.0 | `2:1.0.26-1` | libusb-1.0-0:amd64 | No |
| libva | `2.17.0-1` | libva-drm2:amd64, libva-x11-2:amd64, libva2:amd64 | No |
| libvidstab | `1.1.0-2` | libvidstab1.1:amd64 | No |
| libzstd | `1.5.4+dfsg2-5` | libzstd1:amd64 | No |
| llvm-toolchain-15 | `1:15.0.6-4` | libllvm15:amd64 | No |
| lm-sensors | `1:3.6.0-7.1` | libsensors-config, libsensors5:amd64 | No |
| lz4 | `1.9.4-1` | liblz4-1:amd64 | No |
| mawk | `1.3.4.20200120-3.1` | mawk | No |
| mbedtls | `2.28.3-1` | libmbedcrypto7:amd64 | No |
| mpg123 | `1.31.2-1+deb12u1` | libmpg123-0:amd64 | No |
| nettle | `3.8.1-2` | libhogweed6:amd64, libnettle8:amd64 | Yes |
| numactl | `2.0.16-1` | libnuma1:amd64 | No |
| openal-soft | `1:1.19.1-2` | libopenal-data, libopenal1:amd64 | Yes |
| openssl | `3.0.22-1~deb12u2` | libssl3:amd64, openssl | No |
| p11-kit | `0.24.1-2` | libp11-kit0:amd64 | No |
| pam | `1.5.2-6+deb12u2` | libpam-modules-bin, libpam-modules:amd64, libpam-runtime, libpam0g:amd64 | Yes |
| pango1.0 | `1.50.12+ds-1` | libpango-1.0-0:amd64, libpangocairo-1.0-0:amd64, libpangoft2-1.0-0:amd64 | No |
| perl | `5.36.0-7+deb12u4` | perl-base | Yes |
| pocketsphinx | `0.8+5prealpha+1-15` | libpocketsphinx3:amd64 | No |
| pulseaudio | `16.1+dfsg1-2` | libpulse0:amd64 | No |
| python-pip | `23.0.1+dfsg-1` | python3-pip-whl | No |
| python3-defaults | `3.11.2-1` | libpython3-stdlib:amd64, python3, python3-minimal, python3-venv | No |
| python3-stdlib-extensions | `3.11.2-3` | python3-distutils, python3-lib2to3 | No |
| python3.11 | `3.11.2-6+deb12u9` | libpython3.11-minimal:amd64, libpython3.11-stdlib:amd64, python3.11, python3.11-minimal, python3.11-venv | No |
| readline | `8.2-1.3` | libreadline8:amd64, readline-common | Yes |
| rubberband | `3.1.2+dfsg0-1` | librubberband2:amd64 | No |
| sed | `4.9-1+deb12u1` | sed | Yes |
| shadow | `1:4.13+dfsg1-1+deb12u2` | login, passwd | No |
| shared-mime-info | `2.2-1` | shared-mime-info | No |
| shine | `3.1.1-2` | libshine3:amd64 | No |
| slang2 | `2.3.3-3` | libslang2:amd64 | No |
| speex | `1.2.1-2` | libspeex1:amd64 | No |
| sphinxbase | `0.8+5prealpha+1-16` | libsphinxbase3:amd64 | No |
| sqlite3 | `3.40.1-2+deb12u2` | libsqlite3-0:amd64, sqlite3 | No |
| srt | `1.5.1-1+deb12u2` | libsrt1.5-gnutls:amd64 | No |
| svt-av1 | `1.4.1+dfsg-1` | libsvtav1enc1:amd64 | No |
| systemd | `252.39-1~deb12u2` | libsystemd0:amd64, libudev1:amd64 | No |
| sysvinit | `3.06-4` | sysvinit-utils | Yes |
| tar | `1.34+dfsg-1.2+deb12u1` | tar | Yes |
| twolame | `0.4.0-2` | libtwolame0:amd64 | No |
| usrmerge | `37~deb12u1` | usr-is-merged | No |
| util-linux | `2.38.1-5+deb12u3` | bsdutils, libblkid1:amd64, libmount1:amd64, libsmartcols1:amd64, libuuid1:amd64, mount, util-linux, util-linux-extra | Yes |
| x264 | `2:0.164.3095+gitbaee400-3` | libx264-164:amd64 | No |
| x265 | `3.5-2` | libx265-199:amd64 | No |
| xorg | `1:7.7+23` | x11-common | No |
| xvidcore | `2:1.3.7-1` | libxvidcore4:amd64 | No |
| xxhash | `0.8.1-1` | libxxhash0:amd64 | No |
| xz-utils | `5.4.1-1+deb12u2` | liblzma5:amd64 | Yes |
| z3 | `4.8.12-3.1` | libz3-4:amd64 | No |
| zeromq3 | `4.3.4-6` | libzmq5:amd64 | Yes |
| zimg | `3.0.4+ds1-1` | libzimg2:amd64 | Yes |
| zvbi | `0.2.41-1+deb12u1` | libzvbi-common, libzvbi0:amd64 | No |


## Exact-source acquisition — 2026-11-25

`scripts/collect_distribution_sources.py` acquires all installed Debian source
packages at their recorded source versions, including their Debian patches and
build recipes. It uses authenticated apt source indexes and verifies descriptor
identity and every SHA-256/size in each `.dsc`. It also downloads each installed
Python distribution's exact-version upstream sdist from PyPI, verifies its API
SHA-256/size, and retains the original metadata. Sources are not extracted, built
or executed. Missing versions fail the collection and remain in `source-index.json`.
An upstream Python sdist is not proof of native-wheel or Debian-patch equivalence.

The CI container job runs this larger download only for push commits containing
`[collect-source]` in their message. Normal pull requests and container builds do
not download the source set. Its separate private `runtime-source-evidence-*`
artifact expires after seven days. Before any public binary/image release, retain
and publish the matching source and notice artifacts with that release; an
expiring private CI artifact is not recipient source delivery or a written offer.
No public artifact or image publication is performed by this job.

The latest completed Linux run at `f535afb` passed the BgUtils source-equivalence
check and Debian common-license collection. The new source-collection and Rust
standard-library notice integration still require their own Linux validation.
