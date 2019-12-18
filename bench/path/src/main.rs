extern crate lyon;
#[macro_use]
extern crate bencher;

use lyon::path::{Path, Event, PathEvent, IdEvent, EndpointId, CtrlPointId};
use lyon::path::generic;
use lyon::math::{Point, point};

use bencher::Bencher;

const N: usize = 1;

type GenericPathBuilder = generic::GenericPathBuilder<Point, Point>;

fn simple_path_build_empty(bench: &mut Bencher) {
    bench.iter(|| {
        let mut path = Path::builder();
        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(point(0.0, 0.0));
                for _ in 0..1_000 {
                    path.line_to(point(1.0, 0.0));
                    path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0));
                    path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
                }
                path.close();
            }
        }

        let _ = path.build();
    });
}

fn simple_path_build_prealloc(bench: &mut Bencher) {
    bench.iter(|| {
        let n_points = 60010;
        let n_edges = N * 30_000 + N * 20;
        let mut path = lyon::path::Builder::with_capacity(n_points, n_edges);
        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(point(0.0, 0.0));
                for _ in 0..1_000 {
                    path.line_to(point(1.0, 0.0));
                    path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0));
                    path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
                }
                path.close();
            }
        }

        let _ = path.build();
    });
}

fn generic_build_prealloc(bench: &mut Bencher) {
    bench.iter(|| {
        let n_endpoints = 30010;
        let n_ctrl_points = 30000;
        let n_edges = N * 30_000 + N * 20;

        let mut path: GenericPathBuilder = generic::GenericPathBuilder::with_capacity(
            n_endpoints,
            n_ctrl_points,
            n_edges,
        );

        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(point(0.0, 0.0));
                for _ in 0..1_000 {
                    path.line_to(point(1.0, 0.0));
                    path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0));
                    path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
                }
                path.close();
            }
        }

        let _ = path.build();
    });
}

fn id_only_generic_build_empty(bench: &mut Bencher) {
    bench.iter(|| {
        let mut path = generic::PathCommandsBuilder::new();
        let mut ep = 0;
        let mut cp = 0;
        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(EndpointId(ep));
                ep += 1;
                for _ in 0..1_000 {
                    path.line_to(EndpointId(ep));
                    path.cubic_bezier_to(CtrlPointId(cp), CtrlPointId(cp + 1), EndpointId(ep + 1));
                    path.quadratic_bezier_to(CtrlPointId(cp + 2), EndpointId(ep + 2));
                    cp += 3;
                    ep += 3;
                }
                path.close();
            }
        }

        let _ = path.build();
    });
}

fn generic_build_empty(bench: &mut Bencher) {

    bench.iter(|| {
        let mut path: GenericPathBuilder = generic::GenericPath::builder();
        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(point(0.0, 0.0));
                for _ in 0..1_000 {
                    path.line_to(point(1.0, 0.0));
                    path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0));
                    path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
                }
                path.close();
            }
        }

        let _ = path.build();
    });
}

fn simple_path_iter(bench: &mut Bencher) {
    let mut path = Path::builder();
    for _ in 0..N {
        for _ in 0..10 {
            path.move_to(point(0.0, 0.0));
            for _ in 0..1_000 {
                path.line_to(point(1.0, 0.0));
                path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0));
                path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
            }
            path.close();
        }
    }

    let path = path.build();

    let mut p = point(0.0, 0.0);
    bench.iter(|| {
        for evt in path.iter() {
            p += match evt {
                PathEvent::Begin { at: p }
                | PathEvent::Line { to: p, .. }
                | PathEvent::Quadratic { to: p, .. }
                | PathEvent::Cubic { to: p, .. }
                | PathEvent::End { last: p, .. }
                => {
                    p.to_vector()
                }
            };
        }
    });
}

fn simple_path_id_iter(bench: &mut Bencher) {
    let mut path = Path::builder();
    for _ in 0..N {
        for _ in 0..10 {
            path.move_to(point(0.0, 0.0));
            for _ in 0..1_000 {
                path.line_to(point(1.0, 0.0));
                path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0));
                path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
            }
            path.close();
        }
    }

    let path = path.build();

    let mut i = 0;
    bench.iter(|| {
        for evt in path.id_iter() {
            i += match evt {
                IdEvent::Begin { at: p }
                | IdEvent::Line { to: p, .. }
                | IdEvent::Quadratic { to: p, .. }
                | IdEvent::Cubic { to: p, .. }
                | IdEvent::End { last: p, .. }
                => {
                    p.to_usize()
                }
            };
        }
    });
}

fn generic_id_iter(bench: &mut Bencher) {
    let mut path = generic::PathCommands::builder();
    let mut ep = 0;
    let mut cp = 0;
    for _ in 0..N {
        for _ in 0..10 {
            path.move_to(EndpointId(ep));
            ep += 1;
            for _ in 0..1_000 {
                path.line_to(EndpointId(ep));
                path.cubic_bezier_to(CtrlPointId(cp), CtrlPointId(cp + 1), EndpointId(ep + 1));
                path.quadratic_bezier_to(CtrlPointId(cp + 2), EndpointId(ep + 2));
                cp += 3;
                ep += 3;
            }
            path.close();
        }
    }

    let path = path.build();

    let mut i = 0;
    bench.iter(|| {
        for evt in path.id_events() {
            i += match evt {
                IdEvent::Begin { at: p }
                | IdEvent::Line { to: p, .. }
                | IdEvent::Quadratic { to: p, .. }
                | IdEvent::Cubic { to: p, .. }
                | IdEvent::End { last: p, .. }
                => {
                    p.to_usize()
                }
            };
        }
    });
}

fn no_attrib_iter(bench: &mut Bencher) {

    let path = {
        let mut path = Path::builder_with_attributes(0);
        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(point(0.0, 0.0), &[]);
                for _ in 0..1_000 {
                    path.line_to(point(1.0, 0.0), &[]);
                    path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0), &[]);
                    path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), &[]);
                }
                path.close();
            }
        }

        path.build()
    };

    let mut p = point(0.0, 0.0);
    bench.iter(|| {
        for evt in path.with_attributes() {
            p += match evt {
                Event::Begin { at: p }
                | Event::Line { to: p, .. }
                | Event::Quadratic { to: p, .. }
                | Event::Cubic { to: p, .. }
                | Event::End { last: p, .. }
                => {
                    p.0.to_vector()
                }
            };
        }
    });
}

fn f32x2_attrib_iter(bench: &mut Bencher) {

    let path = {
        let mut path = Path::builder_with_attributes(2);
        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(point(0.0, 0.0), &[0.0, 1.0]);
                for _ in 0..1_000 {
                    path.line_to(point(1.0, 0.0), &[0.0, 1.0]);
                    path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0), &[0.0, 1.0]);
                    path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), &[0.0, 1.0]);
                }
                path.close();
            }
        }

        path.build()
    };

    let mut p = point(0.0, 0.0);
    bench.iter(|| {
        for evt in path.with_attributes() {
            p += match evt {
                Event::Begin { at: p }
                | Event::Line { to: p, .. }
                | Event::Quadratic { to: p, .. }
                | Event::Cubic { to: p, .. }
                | Event::End { last: p, .. }
                => {
                    p.0.to_vector()
                }
            };
        }
    });
}

fn generic_iter(bench: &mut Bencher) {

    let path = {
        let mut path: GenericPathBuilder = generic::GenericPath::builder();
        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(point(0.0, 0.0));
                for _ in 0..1_000 {
                    path.line_to(point(1.0, 0.0));
                    path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0));
                    path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
                }
                path.close();
            }
        }

        path.build()
    };

    let mut p = point(0.0, 0.0);
    bench.iter(|| {
        for evt in path.events() {
            p += match evt {
                Event::Begin { at: p }
                | Event::Line { to: p, .. }
                | Event::Quadratic { to: p, .. }
                | Event::Cubic { to: p, .. }
                | Event::End { last: p, .. }
                => {
                    p.to_vector()
                }
            };
        }
    });
}

fn f32x2_generic_iter(bench: &mut Bencher) {
    struct A { x: f32, y: f32, _z: f32, _w: f32 }
    fn p(x: f32, y: f32) -> A {
        A { x, y, _z: x, _w: y }
    }

    let path = {
        let mut path: generic::GenericPathBuilder<A, Point> = generic::GenericPath::builder();
        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(p(0.0, 0.0));
                for _ in 0..1_000 {
                    path.line_to(p(1.0, 0.0));
                    path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), p(2.0, 2.0));
                    path.quadratic_bezier_to(point(2.0, 0.0), p(2.0, 1.0));
                }
                path.close();
            }
        }

        path.build()
    };

    let mut p: Point = point(0.0, 0.0);
    bench.iter(|| {
        for evt in path.events() {
            p += match evt {
                Event::Begin { at: p }
                | Event::Line { to: p, .. }
                | Event::Quadratic { to: p, .. }
                | Event::Cubic { to: p, .. }
                | Event::End { last: p, .. }
                => {
                    point(p.x, p.y).to_vector()
                }
            };
        }
    });
}

fn generic_points_iter(bench: &mut Bencher) {

    let path = {
        let mut path: GenericPathBuilder = generic::GenericPath::builder();
        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(point(0.0, 0.0));
                for _ in 0..1_000 {
                    path.line_to(point(1.0, 0.0));
                    path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0));
                    path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
                }
                path.close();
            }
        }

        path.build()
    };

    let mut p = point(0.0, 0.0);
    bench.iter(|| {
        for evt in path.events().points() {
            p += match evt {
                PathEvent::Begin { at: p }
                | PathEvent::Line { to: p, .. }
                | PathEvent::Quadratic { to: p, .. }
                | PathEvent::Cubic { to: p, .. }
                | PathEvent::End { last: p, .. }
                => {
                    p.to_vector()
                }
            };
        }
    });
}

fn generic_with_evt_id4_iter(bench: &mut Bencher) {

    let path = {
        let mut path: GenericPathBuilder = generic::GenericPath::builder();
        for _ in 0..N {
            for _ in 0..10 {
                path.move_to(point(0.0, 0.0));
                for _ in 0..1_000 {
                    path.line_to(point(1.0, 0.0));
                    path.cubic_bezier_to(point(2.0, 0.0), point(2.0, 1.0), point(2.0, 2.0));
                    path.quadratic_bezier_to(point(2.0, 0.0), point(2.0, 1.0));
                }
                path.close();
            }
        }

        path.build()
    };

    let mut p = point(0.0, 0.0);
    bench.iter(|| {
        for evt in path.id_events() {
            p += match evt {
                IdEvent::Begin { at } => {
                    path[at].to_vector()
                }
                | IdEvent::Line { to: p, .. }
                | IdEvent::End { last: p, .. }
                => {
                    path[p].to_vector()
                }
                IdEvent::Quadratic { ctrl, to, .. } => {
                    path[ctrl].to_vector() + path[to].to_vector()
                }
                IdEvent::Cubic { ctrl1, ctrl2, to, .. } => {
                    path[ctrl1].to_vector() + path[ctrl2].to_vector() + path[to].to_vector()
                }
            };
        }
    });
}

benchmark_group!(builder,
    simple_path_build_empty,
    simple_path_build_prealloc,
    generic_build_empty,
    id_only_generic_build_empty,
    generic_build_prealloc,
);

benchmark_group!(iter,
    simple_path_iter,
    simple_path_id_iter,
    generic_id_iter,
    generic_iter,
    generic_points_iter,
    generic_with_evt_id4_iter,
    no_attrib_iter,
    f32x2_attrib_iter,
    f32x2_generic_iter,
);

#[cfg(not(feature = "libtess2"))]
benchmark_main!(builder, iter);


