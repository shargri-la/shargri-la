DATA_DIR=data
USER_NUM=10000
END_SLOT=1000
SIMULATE=1

format:
	cargo clean
	cargo clippy --all
	isort visualizer/*.py

visualize: 
	python visualizer/base_fee.py ${DATA_DIR}
	python visualizer/active_user_num.py ${DATA_DIR}
	python visualizer/users.py ${DATA_DIR}
	python visualizer/transaction_num.py ${DATA_DIR}
	python visualizer/mempool.py ${DATA_DIR}

test: 
	cargo run --release -- --user_num ${USER_NUM} --end_slot=${END_SLOT}
	make visualize

ethresearch:

	$(eval DATA_DIR := data/${USER_NUM}_${END_SLOT})
ifeq ($(SIMULATE),1)
	cargo run --release -- --user_num ${USER_NUM} --end_slot=${END_SLOT} --output_dir_path=${DATA_DIR}
endif
	make visualize DATA_DIR=${DATA_DIR}

	$(eval DATA_DIR := data/${USER_NUM}_${END_SLOT}_minimum_0.33)
ifeq ($(SIMULATE),1)
	cargo run --release -- --user_num ${USER_NUM} --end_slot=${END_SLOT} --percentage_of_minimum=0.33 --output_dir_path=${DATA_DIR}
endif
	make visualize DATA_DIR=${DATA_DIR}

	$(eval DATA_DIR := data/${USER_NUM}_${END_SLOT}_minimum_0.67)
ifeq ($(SIMULATE),1)
	cargo run --release -- --user_num ${USER_NUM} --end_slot=${END_SLOT} --percentage_of_minimum=0.67 --output_dir_path=${DATA_DIR}
endif
	make visualize DATA_DIR=${DATA_DIR}

	$(eval DATA_DIR := data/${USER_NUM}_${END_SLOT}_weighted_0.67)
ifeq ($(SIMULATE),1)
	cargo run --release -- --user_num ${USER_NUM} --end_slot=${END_SLOT} --percentage_of_weighted_random=0.67 --output_dir_path=${DATA_DIR}
endif
	make visualize DATA_DIR=${DATA_DIR}

	$(eval DATA_DIR := data/${USER_NUM}_${END_SLOT}_weighted_0.67_popular)
ifeq ($(SIMULATE),1)
	cargo run --release -- --user_num ${USER_NUM} --end_slot=${END_SLOT} --percentage_of_weighted_random=0.67 --popular_user_exists --output_dir_path=${DATA_DIR}
endif
	make visualize DATA_DIR=${DATA_DIR}

	$(eval DATA_DIR := data/${USER_NUM}_${END_SLOT}_minimum_0.33_weighted_0.33)
ifeq ($(SIMULATE),1)
	cargo run --release -- --user_num ${USER_NUM} --end_slot=${END_SLOT} --percentage_of_minimum=0.33 --percentage_of_weighted_random=0.33 --output_dir_path=${DATA_DIR}
endif
	make visualize DATA_DIR=${DATA_DIR}
