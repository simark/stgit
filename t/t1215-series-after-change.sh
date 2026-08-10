#!/bin/sh

test_description='Test stgit.series.show-after-change'

. ./test-lib.sh

test_expect_success 'Initialize stack' '
    stg init &&
    echo "base" >base.txt &&
    git add base.txt &&
    git commit -m "base" &&
    for i in 1 2 3 4; do
        echo "content$i" >"f$i.txt" &&
        git add "f$i.txt" &&
        stg new -m "patch $i" "p$i" &&
        stg refresh || return 1
    done
'

test_expect_success 'Pop shows only the popped patches by default' '
    test_when_finished "stg push -a" &&
    stg pop -n 2 >out 2>&1 &&
    cat >expected <<-\EOF &&
	- p3..p4
	> p2
	EOF
    test_cmp expected out
'

test_expect_success 'Pop shows the whole series' '
    test_when_finished "stg push -a" &&
    test_config stgit.series.show-after-change true &&
    stg pop -n 2 >out 2>&1 &&
    cat >expected <<-\EOF &&
	+ p1
	> p2
	- p3
	- p4
	EOF
    test_cmp expected out
'

test_expect_success 'Push shows the whole series' '
    test_when_finished "stg push -a" &&
    test_config stgit.series.show-after-change true &&
    stg pop -a >/dev/null &&
    stg push >out 2>&1 &&
    cat >expected <<-\EOF &&
	> p1
	- p2
	- p3
	- p4
	EOF
    test_cmp expected out
'

test_expect_success 'Series is shown top-first' '
    test_when_finished "stg push -a" &&
    test_config stgit.series.show-after-change true &&
    test_config stgit.series.top-first true &&
    stg pop -n 2 >out 2>&1 &&
    cat >expected <<-\EOF &&
	- p4
	- p3
	> p2
	+ p1
	EOF
    test_cmp expected out
'

test_expect_success 'New patch is annotated in the series' '
    test_when_finished "stg delete p5" &&
    test_config stgit.series.show-after-change true &&
    stg new -m "patch 5" p5 >out 2>&1 &&
    cat >expected <<-\EOF &&
	+ p1
	+ p2
	+ p3
	+ p4
	> p5 (new)
	EOF
    test_cmp expected out
'

test_expect_success 'Goto shows the whole series' '
    test_when_finished "stg push -a" &&
    test_config stgit.series.show-after-change true &&
    stg goto p2 >out 2>&1 &&
    cat >expected <<-\EOF &&
	+ p1
	> p2
	- p3
	- p4
	EOF
    test_cmp expected out
'

test_expect_success 'Delete shows the deleted patch and the whole series' '
    test_when_finished "stg undo" &&
    test_config stgit.series.show-after-change true &&
    stg delete p2 >out 2>&1 &&
    cat >expected <<-\EOF &&
	# p2
	+ p1
	+ p3
	> p4
	EOF
    test_cmp expected out
'

test_expect_success 'Conflicting push shows the whole series' '
    test_when_finished "stg undo --hard && stg delete pc && stg push -a" &&
    test_config stgit.series.show-after-change true &&
    stg pop -a >/dev/null &&
    echo "conflicting" >f1.txt &&
    git add f1.txt &&
    stg new -m "conflicting patch" pc &&
    stg refresh &&
    conflict stg push p1 >out 2>err &&
    cat >expected <<-\EOF &&
	+ pc
	> p1 (conflict)
	- p2
	- p3
	- p4
	AA f1.txt
	EOF
    test_cmp expected out
'

test_expect_success 'Refresh does not show the series' '
    test_config stgit.series.show-after-change true &&
    echo "more" >>f4.txt &&
    stg refresh >out 2>&1 &&
    ! grep -q "^[+>-] p[123]$" out
'

test_expect_success 'Rename does not show the series' '
    test_when_finished "stg rename q4 p4" &&
    test_config stgit.series.show-after-change true &&
    stg rename p4 q4 >out 2>&1 &&
    cat >expected <<-\EOF &&
	p4 => q4
	> q4
	EOF
    test_cmp expected out
'

test_done
