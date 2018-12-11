import os, sys, time, subprocess, signal

checklist = {
	'waitkill':{'time':15, 'rule':['wait child 1.', 'child 1.', 'child 2.', 'kill parent ok.', 'error: 256 - error 256', 'kill child1 ok.']},
	'sleep':{'time':45, 'rule':[
		'sleep pass.',
		'sleep 1 x 100 slices.',
		'sleep 2 x 100 slices.',
		'sleep 3 x 100 slices.',
		'sleep 4 x 100 slices.',
		'sleep 5 x 100 slices.',
		'sleep 6 x 100 slices.',
		'sleep 7 x 100 slices.',
		'sleep 8 x 100 slices.',
		'sleep 9 x 100 slices.',
		'sleep 10 x 100 slices.',
	]},
	'forktest':{'time':5, 'rule':[
		'forktest pass.',
		'I am child 0',
		'I am child 1',
		'I am child 2',
		'I am child 3',
		'I am child 4',
		'I am child 5',
		'I am child 6',
		'I am child 7',
		'I am child 8',
		'I am child 9',
		'I am child 10',
		'I am child 11',
		'I am child 12',
		'I am child 13',
		'I am child 14',
		'I am child 15',
		'I am child 16',
		'I am child 17',
		'I am child 18',
		'I am child 19',
		'I am child 20',
		'I am child 21',
		'I am child 22',
		'I am child 23',
		'I am child 24',
		'I am child 25',
		'I am child 26',
		'I am child 27',
		'I am child 28',
		'I am child 29',
		'I am child 30',
		'I am child 31',
	]},
	'forktree':{'time':20, 'rule':[
		"I am ''",
		"I am '0'",
		"I am '1'",
		"I am '00'",
		"I am '01'",
		"I am '10'",
		"I am '11'",
		"I am '000'",
		"I am '001'",
		"I am '010'",
		"I am '011'",
		"I am '100'",
		"I am '101'",
		"I am '110'",
		"I am '111'",
		"I am '0000'",
		"I am '0001'",
		"I am '0010'",
		"I am '0011'",
		"I am '0100'",
		"I am '0101'",
		"I am '0110'",
		"I am '0111'",
		"I am '1000'",
		"I am '1001'",
		"I am '1010'",
		"I am '1011'",
		"I am '1100'",
		"I am '1101'",
		"I am '1110'",
		"I am '1111'",
	]},
	'matrix':{'time':10, 'rule':[
		'matrix pass.',
	]},
	'sleepkill':{'time':5, 'rule':[
		'sleepkill pass.',
	]}
}

realpath = sys.path[0]
#os.system(realpath+'/clean_and_make.sh')

for testcase in checklist.keys():
	qemu = subprocess.Popen(realpath+'/make_and_run.sh', shell=True, \
		stdin=subprocess.PIPE, \
		stdout=subprocess.PIPE, \
		stderr = subprocess.PIPE,\
		preexec_fn = os.setsid, close_fds=True \
		)

	time.sleep(5)
	qemu.stdin.write(testcase+'\n')

	time.sleep(checklist[testcase]['time'])
	os.kill(-qemu.pid, 9)
	res = qemu.stdout.read()
	start = res.find(testcase)
	if start < 0 :
		print("Error before go into shell")
	res = res[start:]
	point = 1
	for output in checklist[testcase]['rule']:
		if res.find(output) < 0:
			point = 0
			print(output)
			break
	if point == 1:
		print(testcase+": OK")
	else:
		print(testcase+": ERROR")
		print(res)

