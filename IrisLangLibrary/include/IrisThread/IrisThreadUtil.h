#ifndef _H_IRISTHREADUTIL_
#define _H_IRISTHREADUTIL_

#include "IrisUnil/IrisValue.h"
#include "IrisInterpreter/IrisStructure/IrisContextEnvironment.h"
#include "IrisUnil/IrisStack.h"
#include "IrisUnil/IrisHeap.h"
#include "IrisUnil/IrisStack.h"

#include <unordered_map>
#include <vector>
#include <list>
using namespace std;

struct IrisThreadUniqueInfo {
private:
	typedef unordered_map<string, IrisValue> _ValueMap;
	typedef pair<string, IrisValue> _ValuePair;
	
	typedef vector<IrisContextEnvironment*> _EnvironmentStack;
	typedef vector<unsigned int> _LoopDeepStack;
	typedef vector<IrisValue> _ValueStack;
	typedef vector<bool> _BooleanStack;
	typedef vector<IrisObject*> _ObjectStack;
	typedef unordered_set<IrisContextEnvironment*> _EnvironmentHeap;

public:
	
	//string m_strCurrentFileName = "";
	int m_nCurrentFileIndex = -1;

	//IrisHeap m_hpHeap;
	IrisStack m_stStack;

	_EnvironmentHeap m_ehEnvironmentHeap;
	_EnvironmentStack m_skEnvironmentStack;				      // 线程上下文栈
	_LoopDeepStack m_skDeepStack;							  // 线程深度栈
	_LoopDeepStack m_skMethodDeepStack;					      // 线程方法深度栈
	_ValueStack m_skCounterRegister;						  // 线程Counter栈
	_ValueStack m_skTimerRegister;							  // 线程Timer栈
	_BooleanStack m_skUnimitedLoopFlag;						  // 线程UnitedLoopFlag栈
	_ValueStack m_skVessleRegister;							  // 线程容器栈
	_ValueStack m_skIteratorRegister;						  // 线程Iterator栈
	_ObjectStack m_skTempNewObjectStack;  					  // 线程新生对象临时栈

	IrisValue m_ivResultRegister;							  // 结果寄存器
	IrisValue m_ivCounterRegister;							  // 计数寄存器
	IrisValue m_ivTimerRegister;							  // 次数寄存器
	IrisValue m_ivVessleRegister;							  // for语句迭代容器寄存器
	IrisValue m_ivIteratorRegister;							  // for语句迭代器寄存器
	IrisValue m_ivCompareRegister;							  // switch语句比较寄存器
	list<string> m_lsFieldsRegister;						  // 域寄存器
	bool m_bUnimitedLoopFlagRegister = false;				  // 无限循环标志寄存器
	IrisContextEnvironment* m_pEnvrionmentRegister = nullptr; // 上下文寄存器
	IrisClosureBlock* m_pClosureBlockRegister = nullptr;	  // 块寄存器

	IrisValue m_ivIrregularObjectRegister;					  // 异常状态对象寄存器
	bool m_bIrregularHappenedRegister = false;				  // 异常发生寄存器
	bool m_bFatalErrorHappendRegister = false;			      // FatalError发生寄存器
};

#endif