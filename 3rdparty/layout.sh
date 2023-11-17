#!/bin/bash
 ################################################################
 # $ID: layout.sh       Mon, 30 Oct 2023 15:29:47+0800  mhfan $ #
 #                                                              #
 # Maintainer:  范美辉 (MeiHui FAN) <mhfan@ustc.edu>            #
 # Copyright (c) 2023 M.H.Fan, All rights reserved.             #
 ################################################################

CUR=$(dirname $(readlink -f $0)) && cd $CUR && mkdir -p evg/gpac ftg/stroke
echo "Layout 3rdparty libraries..."

GPAC=~/Devel/gpac
FT2=~/Devel/freetype2
FT2_GIT=https://git.savannah.gnu.org/git/freetype/freetype2.git
FT2_GIT=https://github.com/mhfan/freetype2
GPAC_GIT=https://github.com/gpac/gpac.git
GPAC_GIT=https://github.com/mhfan/gpac # patch to fix for GPAC_FIXED_POINT

[ -d $GPAC ] || { git clone $GPAC_GIT && GPAC=$CUR/gpac; }
[ -d $FT2  ] || { git clone $FT2_GIT && FT2=$CUR/freetype2; }

[ -e ftg/ftgrays.c ] || {
ln -s $FT2/{include/freetype/ftimage.h,src/smooth/ftgrays.[ch],src/raster/ft{misc.h,raster.c}} ftg/;
ln -s $FT2/src/{raster/ftmisc.h,base/ft{stroke,trigon}.c} ftg/stroke/;
ln -s $FT2/include/freetype/ft{stroke,trigon,image}.h ftg/stroke/;
}

[ -e evg/ftgrays.c ] || {
ln -s $GPAC/src/evg/{ftgrays.c,rast_soft.h,stencil.c,surface.c,raster_{argb,rgb,565,yuv}.c,raster3d.c} evg/;
ln -s $GPAC/src/utils/{path2d{,_stroker},math,alloc,color,error}.c evg/; # XXX: constants.c
ln -s $GPAC/include/gpac/{evg,setup,constants,maths,color,path2d,tools,thread}.h evg/gpac/;
touch evg/gpac/{Remotery,config_file,configuration,main,module,version}.h;
}

B2D_GIT=https://github.com/blend2d/blend2d.git
B2D_GIT=https://github.com/mhfan/blend2d # patch to use single precision floating point instead

[ -e blend2d ] || git clone $B2D_GIT
[ -e asmjit  ] || git clone https://github.com/asmjit/asmjit.git

[ -e micro-gl  ] || git clone https://github.com/micro-gl/micro-gl.git

[ -e amanithvg ] || { git clone https://github.com/Mazatech/amanithvg-sdk.git amanithvg &&
    ln -s macosx amanithvg/lib/macos && ln -s ub amanithvg/lib/macos/aarch64; }

 # vim:sts=4 ts=8 sw=4 noet
