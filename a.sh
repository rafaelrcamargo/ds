# Save the cursor position
printf "\033[s"

sleep 1

# Clean and print the argument
cap() { printf "\033[K$1\n"; }

for i in {1..5}; do
  # Restore the cursor position
  printf "\033[u"
  sleep 1

  # Print two lines if the modulo is 0
  if [ $((i % 2)) -eq 0 ]; then
    cap "foooo ${i}"
    cap "bar ${i}"
  else
    cap "foo ${i}"
    cap "baaar ${i}"
  fi

  sleep 0.05
done

sleep 1
printf "\033[u"
sleep 1
cap "Heyy"
sleep 1
cap "Yoo"
sleep 1
