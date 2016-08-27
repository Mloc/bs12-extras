/hook/startup/proc/dmzmq_setup()
	dmzmq_setup()
	return 1
/hook/shutdown/proc/dmzmq_shutdown()
	dmzmq_shutdown()
	return 1

/datum/controller/process/dmzmq

/datum/controller/process/dmzmq/setup()
	name = "dmzmq"
	schedule_interval = 5

/datum/controller/process/dmzmq/doWork()
	callback_socket_pollset.poll()
