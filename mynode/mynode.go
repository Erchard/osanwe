package mynode

var started = false

func start() error {
	var err error

	return err
}

func Required() error {

	var err error
	if started {
		return nil
	} else {
		err = start()
		return err
	}
}
