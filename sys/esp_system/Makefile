COMPONENT=esp_system
INCLUDE_FILES=esp_system.h
INCLUDE_DIR=${IDF_PATH}/components/$(COMPONENT)/include
LIB_FILES=$(patsubst %.h,$(INCLUDE_DIR)/%.h,$(INCLUDE_FILES))
BINDINGS=$(patsubst %.h,src/%.rs,$(INCLUDE_FILES))

.PHONY: force

bindgen: $(BINDINGS)

src/esp_system.rs: $(INCLUDE_DIR)/esp_system.h force
	@bindgen "$(INCLUDE_DIR)/esp_system.h" -o "src/esp_system.rs" \
		--whitelist-function "esp_register_shutdown_handler" \
		--whitelist-function "esp_unregister_shutdown_handler" \
		--whitelist-function "esp_restart" \
		--whitelist-function "esp_reset_reason" \
		--whitelist-function "esp_get_free_heap_size" \
		--whitelist-function "esp_get_free_internal_heap_size" \
		--whitelist-function "esp_get_minimum_free_heap_size" \
		--whitelist-function "esp_random" \
		--whitelist-function "esp_fill_random" \
		--whitelist-function "esp_base_mac_addr_set" \
		--whitelist-function "esp_base_mac_addr_get" \
		--whitelist-function "esp_efuse_mac_get_custom" \
		--whitelist-function "esp_efuse_mac_get_default" \
		--whitelist-function "esp_read_mac" \
		--whitelist-function "esp_derive_local_mac" \
		--whitelist-function "esp_system_abort" \
		--whitelist-function "esp_chip_info" \
		--whitelist-function "soc_has_cache_lock_bug" \
		${BINDGEN_OPTS} \
		-- ${BINDGEN_CLANG_OPTS}

