/****************************************************************
 * $ID: engine_2d.rs  	Thu 02 Nov 2023 11:27:35+0800           *
 *                                                              *
 * Maintainer: 范美辉 (MeiHui FAN) <mhfan@ustc.edu>              *
 * Copyright (c) 2023 M.H.Fan, All rights reserved.             *
 ****************************************************************/

 use criterion::{criterion_group, criterion_main, Criterion};

 fn bench_engine_2d(c: &mut Criterion) {
    use intvg::{tinyvg::TVGImage, render::Render, convert::Convert};
    let mut group = c.benchmark_group("calc24");
    group.sample_size(10);

    let tvg = TVGImage::from_svgd(
        &std::fs::read("data/tiger.svg").unwrap()).unwrap();

    group.bench_function("tiny_skia", |b| b.iter(|| tvg.render(1.0)));
    #[cfg(feature = "evg")] group.bench_function("GPAC/EVG",
        |b| b.iter(|| intvg::render_evg::Render::render(&tvg, 1.0)));
    #[cfg(feature = "b2d")] group.bench_function("Blend2D",
        |b| b.iter(|| intvg::render_b2d::Render::render(&tvg, 1.0)));

    group.finish();
 }

 criterion_group!(benches, bench_engine_2d);
 criterion_main! (benches);

