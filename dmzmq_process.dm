/datum/controller/process/dmzmq

/datum/controller/process/dmzmq/setup()
	name = "dmzmq"
	schedule_interval = 5
	dmzmq_setup()

/datum/controller/process/dmzmq/doWork()
	callback_socket_pollset.poll()
