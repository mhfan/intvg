#!/bin/bash
 ################################################################
 # $ID: layout.sh       一, 30 10 2023 15:29:47+0800  mhfan $ #
 #                                                              #
 # Description:                                                 #
 #                                                              #
 # Maintainer:  范美辉 (MeiHui FAN) <mhfan@ustc.edu>            #
 #                                                              #
 # Copyright (c) 2023 M.H.Fan                                   #
 #   All rights reserved.                                       #
 #                                                              #
 # This file is free software;                                  #
 #   you are free to modify and/or redistribute it  	        #
 #   under the terms of the GNU General Public Licence (GPL).   #
 ################################################################

cd $(dirname $0) && mkdir -p evg/gpac ftg/stroke

GPAC=~/Devel/gpac
FT2=~/Devel/freetype2
FT2_GIT=https://git.savannah.gnu.org/git/freetype/freetype2.git
GPAC_GIT=https://github.com/gpac/gpac.git
GPAC_GIT=https://github.com/mhfan/gpac # patch to fix for GPAC_FIXED_POINT

[ -d $GPAC ] || (git clone $GPAC_GIT && GPAC=gpac)
[ -d $FT2  ] || (git clone $FT2_GIT && FT2=freetype2)

ln -s $FT2/{include/freetype/ftimage.h,src/smooth/ftgrays.[ch],src/ftg/ft{misc.h,raster.c}} ftg/
ln -s $FT2/src/{raster/ftmisc.h,base/ft{stroke,trigon}.c} ftg/stroke/
ln -s $FT2/include/freetype/ft{stroke,trigon,image}.h ftg/stroke/

ln -s $GPAC/src/evg/{ftgrays.c,rast_soft.h,stencil.c,surface.c,raster_{argb,rgb,565,yuv}.c,raster3d.c} evg/
ln -s $GPAC/src/utils/{path2d{,_stroker},math,alloc,color,error}.c evg/ # XXX: constants.c
ln -s $GPAC/include/gpac/{evg,setup,constants,maths,color,path2d,tools,thread}.h evg/gpac/
touch evg/gpac/{Remotery,config_file,configuration,main,module,version}.h

B2D_GIT=https://github.com/blend2d/blend2d.git
B2D_GIT=https://github.com/mhfan/blend2d # patch to use single precision floating point instead

[ -e blend2d ] || git clone $B2D_GIT
[ -e asmjit  ] || git clone https://github.com/asmjit/asmjit.git

[ -e amanithvg ] || git clone https://github.com/Mazatech/amanithvg-sdk.git amanithvg

 # vim:sts=4 ts=8 sw=4 noet
