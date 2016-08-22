/datum/zmq_socket/callback/dnsbl/on_msg(list/msg)
	var/i = findtext(msg[1], ":")
	var/ip = copytext(msg[1], 1, i)
	var/ban_message = copytext(msg[1], i + 1, 0)

	dnsbl_pending -= ip
	dnsbl_bad[ip] = ban_message

	for(var/client/C)
		if(C.address == ip && !C.holder)
			C << "\red<BIG><B>You have been banned due to a blacklist match.\nReason: [ban_message].</B></BIG>"

			log_admin("Server has banned [C.ckey].\nReason: [ban_message]\nThis is a round ban.")
			message_admins("\blue Server has banned [C.ckey].\nReason: [ban_message]\nThis is a round ban.")

			del(C)

/var/disable_blacklist = 0
/var/datum/zmq_socket/callback/dnsbl/dnsbl_sock = null
/var/list/dnsbl_pending = list()
/var/list/dnsbl_bad = list()

/mob/verb/toggle_bl()
	set name = ".toggle_bl"
	set hidden = 1
	if(client.holder)
		disable_blacklist = !disable_blacklist
		src << "disable_blacklist = [disable_blacklist]"

/world/IsBanned(key, address, computer_id)
	if(disable_blacklist || ckey(key) in admin_datums)
		return ..()

	if(!dnsbl_sock)
		dnsbl_sock = new /datum/zmq_socket/callback/dnsbl(ZMQ_DEALER)
		dnsbl_sock.connect("ipc://@dnsbl-dm")
	
	if(address in dnsbl_bad)
		log_access("Failed Login: [key] - Blacklist match")
		message_admins("\blue Failed Login: [key] - Blacklist match")

		return list("reason"="dnsbl match", "desc"="\nReason: [dnsbl_bad[address]]")
	
	if(!(address in dnsbl_pending))
		world.log << address
		dnsbl_pending += address
		dnsbl_sock.send(address)
	
	return ..()
