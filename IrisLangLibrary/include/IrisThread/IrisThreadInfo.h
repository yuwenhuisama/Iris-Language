#ifndef _H_IRISTHREADUTIL_
#define _H_IRISTHREADUTIL_

#include "../IrisCompileConfigure.h"

#include "IrisUnil/IrisValue.h"
#include "IrisInterpreter/IrisStructure/IrisContextEnvironment.h"
#include "IrisUnil/IrisStack.h"
#include "IrisUnil/IrisHeap.h"
#include "IrisUnil/IrisStack.h"
#include "IrisUnil/IrisInternString.h"

#include "IrisInterfaces/IIrisThreadInfo.h"

#include <unordered_map>
#include <vector>
#include <list>
using namespace std;

class IrisThreadInfo : public IIrisThreadInfo {
private:
	typedef unordered_map<IrisInternString, IrisValue, IrisInternString::IrisInerStringHash> _ValueMap;
	typedef pair<IrisInternString, IrisValue> _ValuePair;
	
	typedef vector<IrisContextEnvironment*> _EnvironmentStack;
	typedef vector<unsigned int> _LoopDeepStack;
	typedef vector<IrisObject*> _ObjectStack;
	typedef unordered_set<IrisContextEnvironment*> _EnvironmentHeap;
	typedef unordered_set<IrisObject*> _ObjectHeap;

public:
	
	//string m_strCurrentFileName = "";
	int m_nCurrentFileIndex = -1;

	//IrisHeap m_hpHeap;
	IrisStack m_stStack;

	_EnvironmentHeap m_ehEnvironmentHeap;
	_EnvironmentStack m_skEnvironmentStack;				      // 线程上下文栈

	_ObjectStack m_skTempNewObjectStack;  					  // 线程新生对象临时栈
	_ObjectHeap m_hpObjectInNativeFunctionHeap;				  // 线程本地方法调用生成临时对象栈
	size_t m_nNativeReference = 0;							  // 线程本地方法调用次数统计
	
	IrisValue m_ivResultRegister;							  // 结果寄存器
	IrisValue m_ivCounterRegister;							  // 计数寄存器
	IrisValue m_ivTimerRegister;							  // 次数寄存器
	IrisValue m_ivVessleRegister;							  // for语句迭代容器寄存器
	IrisValue m_ivIteratorRegister;							  // for语句迭代器寄存器
	IrisValue m_ivCompareRegister;							  // switch语句比较寄存器
#if IR_USE_STL_STRING
	list<string> m_lsFieldsRegister;						  // 域寄存器
#else 
	list<IrisInternString> m_lsFieldsRegister;				  // 域寄存器
#endif
	bool m_bUnimitedLoopFlagRegister = false;				  // 无限循环标志寄存器
	IrisContextEnvironment* m_pEnvrionmentRegister = nullptr; // 上下文寄存器
	IrisClosureBlock* m_pClosureBlockRegister = nullptr;	  // 块寄存器

	IrisValue m_ivIrregularObjectRegister;					  // 异常状态对象寄存器
	bool m_bIrregularHappenedRegister = false;				  // 异常发生寄存器
	bool m_bFatalErrorHappendRegister = false;			      // FatalError发生寄存器

	size_t m_nCurrentLineNumber = 0;						  // 当前线程所在代码行号

	IrisThreadInfo() = default;
	~IrisThreadInfo() = default;
};

#endif