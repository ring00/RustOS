#[test]
fn it_works() {
	assert_eq!(2 + 2, 4);
}

#[test]
fn test_event_hub() {
	use event_hub::EventHub;
	let mut eh = EventHub::<usize>::new();
	eh.push(3, 100);
	eh.push(5, 200);
	eh.push(9, 300);
	eh.push(9, 400);
	eh.push(9, 500);
	eh.push(13, 600);
	eh.push(15, 700);
	eh.push(17, 800);
	eh.push(19, 900);

	for _i in 0..3 {
		assert_eq!(eh.pop(), None);
		eh.tick();
	}
	assert_eq!(eh.pop(), Some(100));

	eh.remove(400);

	eh.tick();
	assert_eq!(eh.pop(), None);
	eh.tick();
	assert_eq!(eh.pop(), Some(200));

	for _i in 5..9 {
		assert_eq!(eh.pop(), None);
		eh.tick();
	}
	let data9_1 = eh.pop();
	eh.remove(600);
	let data9_2 = eh.pop();
	let expect9 = (Some(300), Some(500));
	assert!(expect9 == (data9_1, data9_2) || expect9 == (data9_2, data9_1));

	for _i in 9..15 {
		assert_eq!(eh.pop(), None);
		eh.tick();
	}
	eh.tick();
	eh.tick();
	eh.tick();
	assert_eq!(eh.pop(), Some(700));
	assert_eq!(eh.pop(), Some(800));
	assert_eq!(eh.pop(), None);
	eh.tick();
	assert_eq!(eh.pop(), Some(900));
	assert_eq!(eh.pop(), None);
}

#[test]
fn test_rrscheduler() {
	use scheduler::RRScheduler;
	use scheduler::Scheduler;
	let mut rrs = RRScheduler::new(10);
	//assert_eq!(rrs.select(), None);
	rrs.insert(0);
	rrs.insert(1);
	rrs.insert(2);

	assert_eq!(rrs.select(), Some(0));

	rrs.remove(0);
	assert_eq!(rrs.select(), Some(1));
	for _i in 0..7 {
		assert!(!rrs.tick(0));
	}
	assert_eq!(rrs.select(), Some(1));
	rrs.insert(0);
	assert_eq!(rrs.select(), Some(1));


	rrs.remove(1);
	assert_eq!(rrs.select(), Some(2));
	for _i in 0..3 {
		assert!(!rrs.tick(1));
	}
	assert_eq!(rrs.select(), Some(2));
	rrs.insert(1);
	assert_eq!(rrs.select(), Some(2));

	rrs.remove(2);
	assert_eq!(rrs.select(), Some(0));
	for _i in 0..3 {
		assert!(!rrs.tick(2));
	}
	assert_eq!(rrs.select(), Some(0));
	rrs.insert(2);
	assert_eq!(rrs.select(), Some(0));


	rrs.remove(0);
	assert_eq!(rrs.select(), Some(1));
	for _i in 7..9 {
		assert!(!rrs.tick(0));
	}
	assert_eq!(rrs.select(), Some(1));
	assert!(rrs.tick(0));
	assert_eq!(rrs.select(), Some(1));

	rrs.insert(3);
	rrs.insert(4);

	rrs.remove(2);
	assert_eq!(rrs.select(), Some(1));
	for _i in 3..9 {
		assert!(!rrs.tick(2));
	}
	assert_eq!(rrs.select(), Some(1));
	assert!(rrs.tick(2));
	assert_eq!(rrs.select(), Some(1));

	rrs.move_to_head(4);
	assert_eq!(rrs.select(), Some(4));

	rrs.move_to_head(3);
	assert_eq!(rrs.select(), Some(3));

	rrs.insert(0);rrs.remove(0);
	for _i in 0..9 {
		assert!(!rrs.tick(0));
	}
	assert!(rrs.tick(0));
	assert_eq!(rrs.select(), Some(3));
}
