NAME = my_pgp

all: $(NAME)

$(NAME):
	cargo build --release
	cp target/release/my_pgp ./$(NAME)

clean:
	cargo clean

fclean: clean
	rm -f $(NAME)

re: fclean all

.PHONY: all clean fclean re
