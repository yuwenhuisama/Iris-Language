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
	_EnvironmentStack m_skEnvironmentStack;				      // �߳�������ջ
	//_LoopDeepStack m_skDeepStack;								 // �߳����ջ
	//_LoopDeepStack m_skMethodDeepStack;					      // �̷߳������ջ

	_ObjectStack m_skTempNewObjectStack;  					  // �߳�����������ʱջ
	_ObjectHeap m_hpObjectInNativeFunctionHeap;				  // �̱߳��ط�������������ʱ����ջ
	size_t m_nNativeReference = 0;							  // �̱߳��ط������ô���ͳ��
	
	IrisValue m_ivResultRegister;							  // ����Ĵ���
	IrisValue m_ivCounterRegister;							  // �����Ĵ���
	IrisValue m_ivTimerRegister;							  // �����Ĵ���
	IrisValue m_ivVessleRegister;							  // for�����������Ĵ���
	IrisValue m_ivIteratorRegister;							  // for���������Ĵ���
	IrisValue m_ivCompareRegister;							  // switch���ȽϼĴ���
#if IR_USE_STL_STRING
	list<string> m_lsFieldsRegister;						  // ��Ĵ���
#else 
	list<IrisInternString> m_lsFieldsRegister;				  // ��Ĵ���
#endif
	bool m_bUnimitedLoopFlagRegister = false;				  // ����ѭ����־�Ĵ���
	IrisContextEnvironment* m_pEnvrionmentRegister = nullptr; // �����ļĴ���
	IrisClosureBlock* m_pClosureBlockRegister = nullptr;	  // ��Ĵ���

	IrisValue m_ivIrregularObjectRegister;					  // �쳣״̬����Ĵ���
	bool m_bIrregularHappenedRegister = false;				  // �쳣�����Ĵ���
	bool m_bFatalErrorHappendRegister = false;			      // FatalError�����Ĵ���

	int m_nStartDeepRegister = -1;							  // ѭ����ʼ��ȼĴ���
	int m_nEndDeepRegister = -1;							  // ��ǰѭ����ȼĴ���

	size_t m_nCurrentLineNumber = 0;						  // ��ǰ�߳����ڴ����к�

	IrisThreadInfo() = default;
	~IrisThreadInfo() = default;
};

#endif