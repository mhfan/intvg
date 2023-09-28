/*
 *			GPAC - Multimedia Framework C SDK
 *
 *			Authors: Jean Le Feuvre
 *			Copyright (c) Telecom ParisTech 2017-2023
 *					All rights reserved
 *
 *  This file is part of GPAC / filters sub-project
 *
 *  GPAC is free software; you can redistribute it and/or modify
 *  it under the terms of the GNU Lesser General Public License as published by
 *  the Free Software Foundation; either version 2, or (at your option)
 *  any later version.
 *
 *  GPAC is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU Lesser General Public License for more details.
 *
 *  You should have received a copy of the GNU Lesser General Public
 *  License along with this library; see the file COPYING.  If not, write to
 *  the Free Software Foundation, 675 Mass Ave, Cambridge, MA 02139, USA.
 *
 */

// XXX: partial gpac/src/utils/constants.c

#include <gpac/tools.h>
#include <gpac/constants.h>

GF_EXPORT
Bool gf_pixel_get_size_info(GF_PixelFormat pixfmt, u32 width, u32 height, u32 *out_size, u32 *out_stride, u32 *out_stride_uv, u32 *out_planes, u32 *out_plane_uv_height)
{
	u32 stride=0, stride_uv=0, size=0, planes=0, uv_height=0;
	Bool no_in_stride = (!out_stride || (*out_stride==0)) ? GF_TRUE : GF_FALSE;
	Bool no_in_stride_uv = (!out_stride_uv || (*out_stride_uv==0)) ? GF_TRUE : GF_FALSE;

	switch (pixfmt) {
	case GF_PIXEL_GREYSCALE:
		stride = no_in_stride ? width : *out_stride;
		size = stride * height;
		planes=1;
		break;
	case GF_PIXEL_ALPHAGREY:
	case GF_PIXEL_GREYALPHA:
		stride = no_in_stride ? 2*width : *out_stride;
		size = stride * height;
		planes=1;
		break;
	case GF_PIXEL_RGB_444:
	case GF_PIXEL_RGB_555:
	case GF_PIXEL_RGB_565:
		stride = no_in_stride ? 2*width : *out_stride;
		size = stride * height;
		planes=1;
		break;
	case GF_PIXEL_ARGB:
	case GF_PIXEL_RGBA:
	case GF_PIXEL_BGRA:
	case GF_PIXEL_ABGR:
	case GF_PIXEL_RGBX:
	case GF_PIXEL_BGRX:
	case GF_PIXEL_XRGB:
	case GF_PIXEL_XBGR:
	case GF_PIXEL_RGBD:
	case GF_PIXEL_RGBDS:
		stride = no_in_stride ? 4*width : *out_stride;
		size = stride * height;
		planes=1;
		break;
	case GF_PIXEL_RGB_DEPTH:
		stride = no_in_stride ? 3*width : *out_stride;
		stride_uv = no_in_stride_uv ? width : *out_stride_uv;
		size = 4 * width * height;
		planes=1;
		break;
	case GF_PIXEL_RGB:
	case GF_PIXEL_BGR:
		stride = no_in_stride ? 3*width : *out_stride;
		size = stride * height;
		planes=1;
		break;
	case GF_PIXEL_YUV:
	case GF_PIXEL_YVU:
		stride = no_in_stride ? width : *out_stride;
		uv_height = height / 2;
		if (height % 2) uv_height++;
		stride_uv = no_in_stride_uv ? stride/2 : *out_stride_uv;
		if (no_in_stride_uv && (stride%2) )
		 	stride_uv+=1;
		planes=3;
		size = stride * height + stride_uv * uv_height * 2;
		break;
	case GF_PIXEL_YUVA:
	case GF_PIXEL_YUVD:
		stride = no_in_stride ? width : *out_stride;
		uv_height = height / 2;
		if (height % 2) uv_height++;
		stride_uv = no_in_stride_uv ? stride/2 : *out_stride_uv;
		if (no_in_stride_uv && (stride%2) )
		 	stride_uv+=1;
		planes=4;
		size = 2*stride * height + stride_uv * uv_height * 2;
		break;
	case GF_PIXEL_YUV_10:
		stride = no_in_stride ? 2*width : *out_stride;
		uv_height = height / 2;
		if (height % 2) uv_height++;
		stride_uv = no_in_stride_uv ? stride/2 : *out_stride_uv;
		if (no_in_stride_uv && (stride%2) )
		 	stride_uv+=1;
		planes=3;
		size = stride * height + stride_uv * uv_height * 2;
		break;
	case GF_PIXEL_YUV422:
		stride = no_in_stride ? width : *out_stride;
		uv_height = height;
		stride_uv = no_in_stride_uv ? stride/2 : *out_stride_uv;
		if (no_in_stride_uv && (stride%2) )
		 	stride_uv+=1;
		planes=3;
		size = stride * height + stride_uv * height * 2;
		break;
	case GF_PIXEL_YUV422_10:
		stride = no_in_stride ? 2*width : *out_stride;
		uv_height = height;
		stride_uv = no_in_stride_uv ? stride/2 : *out_stride_uv;
		if (no_in_stride_uv && (stride%2) )
		 	stride_uv+=1;
		planes=3;
		size = stride * height + stride_uv * height * 2;
		break;
	case GF_PIXEL_YUV444:
		stride = no_in_stride ? width : *out_stride;
		uv_height = height;
		stride_uv = no_in_stride_uv ? stride : *out_stride_uv;
		planes=3;
		size = stride * height * 3;
		break;
	case GF_PIXEL_YUVA444:
		stride = no_in_stride ? width : *out_stride;
		uv_height = height;
		stride_uv = no_in_stride_uv ? stride : *out_stride_uv;
		planes=4;
		size = stride * height * 4;
		break;
	case GF_PIXEL_YUV444_10:
		stride = no_in_stride ? 2*width : *out_stride;
		uv_height = height;
		stride_uv = no_in_stride_uv ? stride : *out_stride_uv;
		planes=3;
		size = stride * height * 3;
		break;
	case GF_PIXEL_NV12:
	case GF_PIXEL_NV21:
		stride = no_in_stride ? width : *out_stride;
		size = 3 * stride * height / 2;
		uv_height = height/2;
		stride_uv = no_in_stride_uv ? stride : *out_stride_uv;
		planes=2;
		break;
	case GF_PIXEL_NV12_10:
	case GF_PIXEL_NV21_10:
		stride = no_in_stride ? 2*width : *out_stride;
		uv_height = height/2;
		if (height % 2) uv_height++;
		stride_uv = no_in_stride_uv ? stride : *out_stride_uv;
		planes=2;
		size = 3 * stride * height / 2;
		break;
	case GF_PIXEL_UYVY:
	case GF_PIXEL_VYUY:
	case GF_PIXEL_YUYV:
	case GF_PIXEL_YVYU:
		stride = no_in_stride ? 2*width : *out_stride;
		planes=1;
		size = height * stride;
		break;
	case GF_PIXEL_UYVY_10:
	case GF_PIXEL_VYUY_10:
	case GF_PIXEL_YUYV_10:
	case GF_PIXEL_YVYU_10:
		stride = no_in_stride ? 4*width : *out_stride;
		planes=1;
		size = height * stride;
		break;
	case GF_PIXEL_YUV444_PACK:
	case GF_PIXEL_VYU444_PACK:
		stride = no_in_stride ? 3 * width : *out_stride;
		planes=1;
		size = height * stride;
		break;
	case GF_PIXEL_YUVA444_PACK:
	case GF_PIXEL_UYVA444_PACK:
		stride = no_in_stride ? 4 * width : *out_stride;
		planes=1;
		size = height * stride;
		break;
	case GF_PIXEL_YUV444_10_PACK:
		stride = no_in_stride ? 4 * width : *out_stride;
		planes = 1;
		size = height * stride;
		break;

	case GF_PIXEL_GL_EXTERNAL:
		planes = 1;
		size = 0;
		stride = 0;
		stride_uv = 0;
		uv_height = 0;
		break;
	case GF_PIXEL_V210:
		if (no_in_stride) {
			stride = width;
			while (stride % 48) stride++;
			stride = stride * 16 / 6; //4 x 32 bits to represent 6 pixels
		} else {
			stride = *out_stride;
		}
		planes=1;
		size = height * stride;
		break;
	default:
		GF_LOG(GF_LOG_ERROR, GF_LOG_CORE, ("Unsupported pixel format %s, cannot get size info\n", gf_pixel_fmt_name(pixfmt) ));
		return GF_FALSE;
	}
	if (out_size) *out_size = size;
	if (out_stride) *out_stride = stride;
	if (out_stride_uv) *out_stride_uv = stride_uv;
	if (out_planes) *out_planes = planes;
	if (out_plane_uv_height) *out_plane_uv_height = uv_height;
	return GF_TRUE;
}

