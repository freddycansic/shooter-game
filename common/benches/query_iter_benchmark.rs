use common::ecs::system_parameters::query::Query;
use common::engine::scheduler::Scheduler;
use common::world::World;
use common_macros::Component;
use criterion::Criterion;

#[derive(Component)]
struct A(u32);

pub fn query_iter_benchmark(c: &mut Criterion) {
    let mut world = World::default();

    fn query_iter(mut q: Query<&A>) {
        let sum = q.iter().map(|a| a.0).sum::<u32>();
    }

    let mut scheduler = Scheduler::default();
    scheduler.register(query_iter);

    for magnitude in 3..8 {
        let num_previous_components = 10_u32.pow(magnitude - 1);
        let num_total_components = 10_u32.pow(magnitude);

        for i in num_previous_components..num_total_components {
            world.spawn(A(i));
        }

        let bench_name = format!("query_iter_{}", num_total_components);

        c.bench_function(&bench_name, |b| b.iter(|| scheduler.run_systems(&mut world)));
    }
}
